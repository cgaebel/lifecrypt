mod cmdline;
mod crypt;
mod editor;

use anyhow::{Context, Result};
use cmdline::Opts;
use std::fs;
use std::path::PathBuf;

fn load_encrypted_file(file: &PathBuf) -> Result<crypt::Encrypted> {
  let binary_file_contents = fs::read(file)?;
  let as_json: crypt::EncryptedSerializable =
    serde_json::from_str(&String::from_utf8(binary_file_contents)?)?;
  as_json.to_encrypted()
}

fn view(file: &PathBuf) -> Result<()> {
  let encrypted = load_encrypted_file(file)?;
  let password = rpassword::prompt_password_stdout("Password: ")?;
  let decrypted_contents = crypt::decrypt(encrypted, &password);
  print!("{}", String::from_utf8(decrypted_contents)?);
  Ok(())
}

fn write_encrypted_file(
  file: &PathBuf,
  encrypted: &crypt::Encrypted,
) -> Result<()> {
  let jsonable = crypt::EncryptedSerializable::new(encrypted);
  let json = serde_json::to_string_pretty(&jsonable)?;
  fs::write(file, &json)?;
  return Ok(());
}

fn edit(file: &PathBuf) -> Result<()> {
  let encrypted = load_encrypted_file(file)
    .with_context(|| format!("loading encrypted file {:?}", file))?;
  let password = rpassword::prompt_password_stdout("Password: ")?;
  let decrypted_contents = crypt::decrypt(encrypted, &password);
  let edited_contents = editor::spawn(&decrypted_contents)
    .with_context(|| format!("editing file {:?}", file))?;
  let newly_encrypted = crypt::encrypt(&edited_contents, &password);
  write_encrypted_file(file, &newly_encrypted)
}

fn main() {
  let what_to_do = cmdline::parse();
  let result = match what_to_do {
    Opts::View { file } => view(&file),
    Opts::Edit { file } => edit(&file),
  };
  result.unwrap();
}
