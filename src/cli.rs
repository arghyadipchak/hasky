use std::{fs, num::ParseIntError, path::PathBuf, sync::Arc, time::Duration};

use clap::Parser;

pub const OUTFILE_SUFFIX: &str = "_graded";

#[derive(Parser)]
pub struct Cli {
  #[arg(short, long, help = "Input file (moody compatible)")]
  pub input_file: PathBuf,

  #[arg(short, long, help = format!("Output file [default: {{input-file}}{OUTFILE_SUFFIX}.yml]"))]
  output_file: Option<PathBuf>,

  #[arg(
    short,
    long,
    help = "Maximum grade (equally divided among testcases) [default: assignment max grade]"
  )]
  pub max_grade: Option<f32>,

  #[arg(short, long, help = "Don't mark as graded")]
  pub not_graded: bool,

  #[arg(
    short = 'f',
    long,
    help = "File containing testcases helper functions"
  )]
  test_functions: Option<PathBuf>,

  #[arg(short = 'c', long, help = "File containing testcases")]
  test_cases: PathBuf,

  #[arg(short, long, value_parser = parse_duration, help="Timeout for each test case (in secs)")]
  pub timeout: Option<Duration>,

  #[arg(short, long, default_value_t = 1, help = "No of workers to run")]
  pub workers: usize,
}

fn parse_duration(arg: &str) -> Result<Duration, ParseIntError> {
  Ok(Duration::from_secs(arg.parse()?))
}

impl Cli {
  pub fn get_output_file(&self) -> PathBuf {
    if let Some(file) = &self.output_file {
      file.clone()
    } else {
      let mut fname = self
        .input_file
        .file_stem()
        .unwrap_or_default()
        .to_os_string();
      fname.push(OUTFILE_SUFFIX);

      self
        .input_file
        .with_file_name(fname)
        .with_extension(self.input_file.extension().unwrap_or_default())
    }
  }

  pub fn get_testcases(&self) -> Vec<Arc<String>> {
    fs::read_to_string(&self.test_cases)
      .unwrap_or_default()
      .lines()
      .map(str::trim)
      .filter(|t| !(t.is_empty() || t.starts_with("--")))
      .map(|s| Arc::new(s.to_string()))
      .collect::<Vec<_>>()
  }

  pub fn get_test_functions(&self) -> Arc<String> {
    Arc::new(
      self
        .test_functions
        .clone()
        .map_or(String::new(), |p| fs::read_to_string(p).unwrap_or_default()),
    )
  }
}
