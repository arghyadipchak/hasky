mod cli;
mod code;

use std::{
  collections::HashMap,
  fmt,
  fs::{self, File},
  io::Write,
  path::PathBuf,
  process,
  sync::{mpsc, Arc},
};

use clap::Parser;
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use threadpool::ThreadPool;

use cli::Cli;
use code::{Code, Status};

const HASKY_DISCLAIMER: &str =
  "# Graded by Hasky (https://github.com/arghyadipchak/hasky)";

fn unwrap_or_exit<T, E: fmt::Display>(result: Result<T, E>) -> T {
  match result {
    Ok(val) => val,
    Err(err) => {
      eprintln!("{err}");
      process::exit(1);
    }
  }
}

#[derive(Deserialize, Serialize)]
struct Assignment {
  id: u64,
  name: String,
  max_grade: f32,
  submissions: Vec<Submission>,
}

#[derive(Deserialize, Serialize)]
struct Submission {
  user: User,
  graded: bool,
  late: u64,
  grade: f32,
  feedback: String,
  files: Vec<PathBuf>,
}

#[derive(Deserialize, Serialize)]
struct User {
  id: u64,
  fullname: Value,
  email: String,
}

impl Status {
  fn get_emoji(&self) -> &str {
    match self {
      Self::Passed => "✔️",
      Self::Failed => "❌",
      Self::Timeout => "⌛",
    }
  }
}

fn main() {
  let cli = Cli::parse();

  let mut assignment =
    unwrap_or_exit(serde_yaml::from_reader::<_, Assignment>(unwrap_or_exit(
      File::open(&cli.file),
    )));

  let test_cases = fs::read_to_string(cli.test_cases)
    .unwrap_or_default()
    .lines()
    .map(str::trim)
    .filter(|t| !(t.is_empty() || t.starts_with("--")))
    .map(|s| Arc::new(s.to_string()))
    .collect::<Vec<_>>();
  let n = test_cases.len();

  let bar = ProgressBar::new(0);
  let test_functions = Arc::new(
    cli
      .test_functions
      .map_or(String::new(), |p| fs::read_to_string(p).unwrap_or_default()),
  );

  let mut user_test_cases = HashMap::new();
  let pool = ThreadPool::new(cli.workers);
  let (tx, rx) = mpsc::channel();

  for submission in &mut assignment.submissions {
    if submission.graded {
      continue;
    }

    let code = Arc::new(Code::new(&submission.files));
    user_test_cases.insert(submission.user.id, vec![Status::Failed; n]);
    bar.inc_length(n as u64);

    for (i, test_case) in test_cases.iter().enumerate() {
      let tx = tx.clone();
      let code = code.clone();
      let test_case = test_case.clone();
      let uid = submission.user.id;
      let test_functions = test_functions.clone();

      pool.execute(move || {
        let t = code.check(&test_functions, &test_case, cli.timeout);

        tx.send((uid, i, t)).unwrap();
      });
    }
  }

  drop(tx);

  for (uid, tid, status) in &rx {
    if let Some(v) = user_test_cases.get_mut(&uid) {
      v[tid] = status;
    }
    bar.inc(1);
  }

  let max_grade = cli.max_grade.unwrap_or(assignment.max_grade);

  for submission in &mut assignment.submissions {
    if submission.graded {
      continue;
    }

    if !submission.feedback.is_empty() {
      submission.feedback.push('\n');
    }

    if let Some(user_test_case) = user_test_cases.get(&submission.user.id) {
      let feedback = user_test_case
        .iter()
        .zip(test_cases.iter())
        .fold(String::new(), |out, (s, t)| {
          out + t + " " + s.get_emoji() + "\n"
        });
      submission.feedback.push_str(feedback.trim());

      let passed = user_test_case
        .iter()
        .filter(|tc| matches!(tc, Status::Passed))
        .count();
      submission.grade +=
        (passed as f32 * max_grade * 100.0 / n as f32).round() / 100.0;

      if !cli.no_grade {
        submission.graded = true;
      }
    }
  }

  let mut fh =
    unwrap_or_exit(File::create(cli.output_file.unwrap_or(cli.file)));

  unwrap_or_exit(writeln!(fh, "{HASKY_DISCLAIMER}\n"));
  unwrap_or_exit(serde_yaml::to_writer(fh, &assignment));
}
