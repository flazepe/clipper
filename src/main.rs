mod clipper;
mod ffmpeg;

pub use clipper::Clipper;

pub fn main() {
    Clipper::from_args().run();
}
