mod args;
mod functions;

use args::Args;
use clap::Parser;
use functions::spawn_ffmpeg;

fn main() {
    spawn_ffmpeg(Args::parse());
}
