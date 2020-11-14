mod cmdline;

fn main() {
    let opts = cmdline::parse();
    println!("{:?}", opts);
}
