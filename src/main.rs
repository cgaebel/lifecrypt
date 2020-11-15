mod cmdline;
mod crypt;
mod editor;

use anyhow::{ensure, Context, Result};
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
  contents: &[u8],
  password: &str,
) -> Result<()> {
  let encrypted = crypt::encrypt(contents, password)?;
  let json = encrypted.to_json()?;
  Ok(fs::write(file, &json)?)
}

fn get_decrypted_contents(file: &PathBuf, password: &str) -> Result<Vec<u8>> {
  if file.exists() {
    let encrypted = load_encrypted_file(file)
      .with_context(|| format!("loading encrypted file {:?}", file))?;
    let decrypted_contents = crypt::decrypt(encrypted, &password)
      .with_context(|| format!("decrypting {:?}", file))?;
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
    ensure!(confirmed_password == password, "passwords don't match");
  }
  let decrypted_contents = get_decrypted_contents(file, &password)?;
  let edited_contents = editor::spawn(&decrypted_contents)
    .with_context(|| format!("editing {:?}", file))?;
  write_encrypted_file(file, &edited_contents, &password)
    .with_context(|| format!("writing new encrypted file {:?}", file))
}

fn change_password(file: &PathBuf) -> Result<()> {
  let current_password =
    rpassword::prompt_password_stdout("current password: ")?;
  let decrypted_contents = get_decrypted_contents(file, &current_password)?;
  let new_password = rpassword::prompt_password_stdout("new password: ")?;
  let confirmed_password =
    rpassword::prompt_password_stdout("confirm password: ")?;
  ensure!(confirmed_password == new_password, "passwords don't match");
  write_encrypted_file(file, &decrypted_contents, &new_password)
}

fn main() -> Result<()> {
  match cmdline::parse() {
    Opts::View { file } => view(&file),
    Opts::Edit { file } => edit(&file),
    Opts::ChangePassword { file } => change_password(&file),
  }
}
