//! Parses command line arguments.
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "encrypt your life")]
pub enum Opts {
    #[structopt(about = "Edit the contents of a lifecrypt vault.")]
    Edit {
        #[structopt(name = "FILE")]
        file: String,
    },
    #[structopt(about = "Print the contents of a lifecrypt vault to stdout.")]
    View {
        #[structopt(name = "FILE")]
        file: String,
    },
}

pub fn parse() -> Opts {
    Opts::from_args()
}
