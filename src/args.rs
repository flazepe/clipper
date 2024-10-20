use clap::Parser;

/// A simple ffmpeg wrapper for clipping videos
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The input file
    #[arg(short, long)]
    pub input: String,

    /// The segment ranges (e.g. "2:00-2:30 9:20-9:30"). The space can be replaced by other supported delimiters. A simple duration like "1:25" can also be used to act as a skip (equivalent to ffmpeg's -ss option)
    #[arg(short, long)]
    pub segments: String,

    /// The cq, if using NVENC
    #[arg(short, long)]
    pub cq: Option<String>,

    /// Whether to convert to HEVC/H.265 instead of AVC/H.264
    #[arg(long, action)]
    pub hevc: bool,

    /// Whether to mute the entire video
    #[arg(short, long, action)]
    pub mute: bool,

    /// Whether to fade between segments
    #[arg(short, long, action)]
    pub fade: bool,

    /// Whether to debug
    #[arg(short, long, action)]
    pub debug: bool,

    /// The output file
    pub output: String,
}
