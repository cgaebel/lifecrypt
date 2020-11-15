mod cmdline;
mod crypt;
mod editor;

use anyhow::{bail, Context, Result};
use cmdline::Opts;
use std::fs;
use std::path::PathBuf;

fn load_encrypted_file(file: &PathBuf) -> Result<crypt::Encrypted> {
  let binary_file_contents = fs::read(file)?;
  let str_file_contents = String::from_utf8(binary_file_contents)?;
  crypt::Encrypted::from_json(&str_file_contents)
}

fn view(file: &PathBuf) -> Result<()> {
  let encrypted = load_encrypted_file(file)?;
  let password = rpassword::prompt_password_stdout("file password: ")?;
  let decrypted_contents = crypt::decrypt(encrypted, &password)?;
  print!("{}", String::from_utf8(decrypted_contents)?);
  Ok(())
}

fn write_encrypted_file(
  file: &PathBuf,
  encrypted: &crypt::Encrypted,
) -> Result<()> {
  let json = encrypted.to_json()?;
  fs::write(file, &json)?;
  return Ok(());
}

fn get_decrypted_contents(file: &PathBuf, password: &str) -> Result<Vec<u8>> {
  if file.exists() {
    let encrypted = load_encrypted_file(file)
      .with_context(|| format!("loading encrypted file {:?}", file))?;
    let decrypted_contents = crypt::decrypt(encrypted, &password)?;
    Ok(decrypted_contents)
  } else {
    Ok(vec![])
  }
}

fn edit(file: &PathBuf) -> Result<()> {
  let password = rpassword::prompt_password_stdout("file password: ")?;
  if !file.exists() {
    println!("Creating a new crypt...");
    let confirmed_password =
      rpassword::prompt_password_stdout("confirm password: ")?;
    if confirmed_password != password {
      bail!("passwords don't match");
    }
  }
  let decrypted_contents = get_decrypted_contents(file, &password)?;
  let edited_contents = editor::spawn(&decrypted_contents)
    .with_context(|| format!("editing file {:?}", file))?;
  let newly_encrypted = crypt::encrypt(&edited_contents, &password)?;
  write_encrypted_file(file, &newly_encrypted)
}

fn main() -> Result<()> {
  match cmdline::parse() {
    Opts::View { file } => view(&file),
    Opts::Edit { file } => edit(&file),
  }
}
