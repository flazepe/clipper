use clipper::Clipper;
use std::process::exit;

fn main() {
    if let Err(error) = Clipper::from_env_args().and_then(|clipper| clipper.run()) {
        println!("\x1b[38;5;203m{error}\x1b[0m");
        exit(1);
    }
}
