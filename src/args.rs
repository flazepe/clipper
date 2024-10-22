use crate::error;
use std::env::args as env_args;

/// A simple ffmpeg wrapper for clipping videos.
pub struct Args {
    /// The input files
    pub input: Vec<(String, Vec<String>)>,

    /// Whether to fade between segments. If set (e.g. "-fade=1"), this would be the fade duration in secs (default: 0.5)
    pub fade: Option<f64>,

    /// The CQ, if using NVENC
    pub cq: Option<String>,

    /// Whether to convert to HEVC/H.265 instead of AVC/H.264
    pub hevc: bool,

    /// Whether to disable the video track
    pub no_video: bool,

    /// Whether to disable the audio track
    pub no_audio: bool,

    /// Whether to dry run
    pub dry_run: bool,

    /// The output file
    pub output: String,
}

impl Args {
    pub fn parse() -> Self {
        let mut input = vec![];
        let mut fade = None;
        let mut cq = None;
        let mut hevc = false;
        let mut no_video = false;
        let mut no_audio = false;
        let mut dry_run = false;
        let mut output = None;

        let args = env_args().skip(1).collect::<Vec<String>>();
        let mut current_option = None::<String>;

        for (index, arg) in args.iter().enumerate() {
            let arg = arg.clone();

            if arg.starts_with('-') {
                let option = arg.trim_start_matches('-').split('=').next().unwrap_or("");

                match option {
                    "input" | "i" => current_option = Some(option.into()),
                    "segment" | "s" => current_option = Some(option.into()),
                    "fade" | "f" => {
                        fade = arg
                            .split('=')
                            .last()
                            .map(|fade| fade.parse::<f64>().unwrap_or(0.5));
                    }
                    "cq" => current_option = Some(option.into()),
                    "hevc" => hevc = true,
                    "no-video" | "vn" => no_video = true,
                    "no-audio" | "an" => no_audio = true,
                    "dry-run" | "d" => dry_run = true,
                    _ => error!(format!("Invalid option: -{option}")),
                }

                continue;
            }

            if let Some(option) = &current_option {
                match option.as_str() {
                    "input" | "i" => input.push((arg, vec![])),
                    "segment" | "s" => {
                        if let Some((_, segments)) = input.last_mut() {
                            segments.push(arg);
                        }
                    }
                    "cq" => cq = Some(arg),
                    _ => {}
                }

                current_option = None;
            } else if index == args.len() - 1 {
                output = Some(arg);
            }
        }

        // Validations
        if input.is_empty() {
            error!("Please enter at least one input.");
        }

        if let Some((input, _)) = input.iter().find(|input| input.1.is_empty()) {
            error!(format!(r#"Input "{input}" has no segments!"#));
        }

        if cq.as_ref().map_or(false, |cq| cq.parse::<f64>().is_err()) {
            error!("Invalid CQ value.");
        }

        if no_video && no_audio {
            error!("Video and audio track cannot be disabled at the same time.");
        }

        let Some(output) = output else {
            error!("Please specify an output file.");
        };

        Self {
            input,
            fade,
            cq,
            hevc,
            no_video,
            no_audio,
            dry_run,
            output,
        }
    }
}
