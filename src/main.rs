mod args;
mod clipper;

use args::Args;
use clipper::Clipper;

fn main() {
    Clipper::new(Args::parse()).run();
}
