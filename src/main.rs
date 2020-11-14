mod cmdline;
mod editor;

fn main() {
    let opts = cmdline::parse();
    println!("{:?}", opts);
}
