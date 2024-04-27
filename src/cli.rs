use std::{num::ParseIntError, path::PathBuf, time::Duration};

use clap::Parser;

#[derive(Parser)]
pub struct Cli {
  #[arg(short, long)]
  pub file: PathBuf,

  #[arg(short, long)]
  pub output_file: Option<PathBuf>,

  #[arg(short, long)]
  pub max_grade: Option<f32>,

  #[arg(short = 'g', long)]
  pub no_grade: bool,

  #[arg(short = 'u', long)]
  pub test_functions: Option<PathBuf>,

  #[arg(short = 'c', long)]
  pub test_cases: PathBuf,

  #[arg(short, long, value_parser = parse_duration)]
  pub timeout: Option<Duration>,

  #[arg(short, long, default_value_t = 1)]
  pub workers: usize,
}

fn parse_duration(arg: &str) -> Result<Duration, ParseIntError> {
  Ok(Duration::from_secs(arg.parse()?))
}
