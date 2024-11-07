use std::process::exit;

use clipper::Clipper;

fn main() {
    if let Err(error) = Clipper::from_env_args().and_then(|clipper| clipper.run()) {
        println!("\x1b[38;5;203m{error}\x1b[0m");
        exit(1);
    }
}
