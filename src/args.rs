use clap::{ArgAction, Parser};

/// A simple ffmpeg wrapper for clipping videos
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The input file
    #[arg(short, long)]
    pub input: String,

    /// The segment duration range to add (e.g. "--segment 2:00-2:30"). This option can be repeated to add more segments
    #[arg(short, long = "segment", id = "DURATION RANGE")]
    pub segments: Vec<String>,

    /// The CQ, if using NVENC
    #[arg(short, long)]
    pub cq: Option<String>,

    /// Whether to convert to HEVC/H.265 instead of AVC/H.264
    #[arg(short = 'e', long)]
    pub hevc: bool,

    /// Whether to mute the entire video
    #[arg(short, long)]
    pub mute: bool,

    /// Whether to fade between segments. If set (e.g. "--fade=1"), this would be the fade duration in secs (default: 0.5)
    #[arg(short, long, num_args = 0..=1, require_equals = true, default_missing_value = "0.5")]
    pub fade: Option<f64>,

    /// Whether to debug
    #[arg(short, long)]
    pub debug: bool,

    /// The output file
    pub output: String,
}
