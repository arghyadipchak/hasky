use std::{
  fs,
  path::Path,
  process::{Command, Stdio},
  time::Duration,
};

use lazy_static::lazy_static;
use regex::Regex;
use wait_timeout::ChildExt;

pub struct Code {
  source_codes: Vec<String>,
}

lazy_static! {
  static ref IMPORTS_REGEX: Regex = Regex::new(r"^\s*(import.*\s*)*").unwrap();
  static ref MODULE_REGEX: Regex =
    Regex::new(r"module\s+\w+\s+(\(.*\)\s+)?where").unwrap();
  static ref MAIN_REGEX: Regex =
    Regex::new(r"(main\s+::\s+IO\s+\(\)\s+)?main\s+=\s+.*(?:\n\s.*)*").unwrap();
}

impl Code {
  pub fn new<P>(files: impl IntoIterator<Item = P>) -> Self
  where
    P: AsRef<Path>,
  {
    let mut source_codes = Vec::new();
    for file in files {
      let content = fs::read_to_string(file).unwrap_or_default();

      let content = MODULE_REGEX.replace_all(&content, "");
      let content = MAIN_REGEX.replace_all(&content, "");

      if content.is_empty() {
        continue;
      }

      source_codes.push(content.to_string());
    }

    Self { source_codes }
  }
}

#[derive(Clone)]
pub enum Status {
  Failed,
  Passed,
  Timeout,
}

impl Code {
  pub fn check(
    &self,
    test_functions: &str,
    test_case: &str,
    timeout: Option<Duration>,
  ) -> Status {
    let dir = &tempfile::tempdir().unwrap();
    let p = test_functions
      .rfind("import")
      .and_then(|i| test_functions[i..].find('\n'))
      .map(|i| i + 1)
      .unwrap_or_default();

    for source_code in &self.source_codes {
      let fpath = dir.path().join("test.hs");

      if fs::write(
        &fpath,
        format!(
          "import System.Exit (exitFailure, exitSuccess)\n\
          {}\n\
          {source_code}\n\n\
          {}\n\n\
          main :: IO ()\n\
          main = if ({test_case}) then exitSuccess else exitFailure",
          &test_functions[..p],
          &test_functions[p..]
        ),
      )
      .is_err()
      {
        continue;
      }

      if Command::new("ghc")
        .arg("-O2")
        .arg(&fpath)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut child| child.wait().map(|es| !es.success()))
        .unwrap_or(true)
      {
        continue;
      }

      if let Ok(mut child) = Command::new("./test")
        .current_dir(dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
      {
        if let Some(d) = timeout {
          if let Some(es) = child.wait_timeout(d).unwrap() {
            if es.success() {
              return Status::Passed;
            }
          } else {
            child.kill().unwrap();
            return Status::Timeout;
          }
        } else if child.wait().unwrap().success() {
          return Status::Passed;
        }
      }
    }

    Status::Failed
  }
}
