mod cmdline;
mod editor;

use cmdline::Opts;
use std::path::PathBuf;

fn view(file: &PathBuf) {
    println!("viewing {:?}", file);
}

fn edit(file: &PathBuf) {
    println!("editing {:?}", file);
}

fn main() {
    let what_to_do = cmdline::parse();
    match what_to_do {
        Opts::View { file } => view(&file),
        Opts::Edit { file } => edit(&file),
    }
}
