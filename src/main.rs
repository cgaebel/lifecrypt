mod cmdline;
mod editor;

use anyhow::{Context, Result};
use cmdline::Opts;
use std::fs;
use std::path::PathBuf;
use std::str;

fn view(file: &PathBuf) -> Result<()> {
  let binary_file_contents = fs::read(file)?;
  let str_file_contents = str::from_utf8(&binary_file_contents)?;
  print!("{}", str_file_contents);
  Ok(())
}

fn edit(file: &PathBuf) -> Result<()> {
  let binary_file_contents = fs::read(file)?;
  let edited_contents = editor::spawn(&binary_file_contents)
    .with_context(|| format!("editing file {:?}", file))?;
  fs::write(file, edited_contents)?;
  Ok(())
}

fn main() {
  let what_to_do = cmdline::parse();
  let result = match what_to_do {
    Opts::View { file } => view(&file),
    Opts::Edit { file } => edit(&file),
  };
  result.unwrap()
}
