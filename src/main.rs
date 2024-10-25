mod clipper;
mod ffmpeg;

use clipper::Clipper;

fn main() {
    Clipper::new().run();
}
