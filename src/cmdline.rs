//! Parses command line arguments.
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "encrypt your life")]
pub enum Opts {
  #[structopt(about = "Edit the contents of a lifecrypt vault.")]
  Edit {
    #[structopt(name = "FILE")]
    file: PathBuf,
  },
  #[structopt(about = "Print the contents of a lifecrypt vault to stdout.")]
  View {
    #[structopt(name = "FILE")]
    file: PathBuf,
  },
  #[structopt(about = "Change the password to a lifecrypt vault.")]
  ChangePassword {
    #[structopt(name = "FILE")]
    file: PathBuf,
  },
}

pub fn parse() -> Opts {
  Opts::from_args()
}
