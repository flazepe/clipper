use crate::{
    error,
    ffmpeg::{Encoder, Inputs, Output},
};
use std::{
    env::args,
    process::{exit, Command},
    vec::IntoIter,
};

pub struct Clipper {
    inputs: Inputs,
    encoder: Encoder,
    output: Output,
    dry_run: bool,
}

impl Clipper {
    pub fn new() -> Self {
        let mut inputs = Inputs::default();
        let mut encoder = Encoder::default();
        let mut output = Output::default();
        let mut dry_run = false;

        // The current option for parsing args
        let mut current_option = None::<String>;

        for arg in args().skip(1) {
            let arg = arg.clone();

            if arg.starts_with('-') {
                let option = arg.trim_start_matches('-').split('=').next().unwrap_or("");

                match option {
                    "input" | "i" => current_option = Some(option.into()),
                    "video-track" | "vt" => current_option = Some(option.into()),
                    "audio-track" | "at" => current_option = Some(option.into()),
                    "subtitle-track" | "st" => current_option = Some(option.into()),
                    "speed" | "spd" => current_option = Some(option.into()),
                    "segment" | "s" => current_option = Some(option.into()),
                    "fade" | "f" => inputs.set_fade(arg),
                    "nvenc" => encoder.set_nvenc(true),
                    "hevc" => encoder.set_hevc(true),
                    "preset" | "p" => current_option = Some(option.into()),
                    "crf" => current_option = Some(option.into()),
                    "cq" => current_option = Some(option.into()),
                    "no-video" | "vn" => inputs.set_no_video(true),
                    "no-audio" | "an" => inputs.set_no_audio(true),
                    "dry-run" | "d" => dry_run = true,
                    "help" | "h" => Self::print_help(),
                    "version" | "v" => Self::print_version(),
                    _ => error!("Invalid option: -{option}. Use -help for more information."),
                }

                continue;
            }

            if let Some(option) = &current_option {
                match option.as_str() {
                    "input" | "i" => inputs.add_input(arg),
                    "video-track" | "vt" => {
                        if let Some(last_input) = inputs.get_last_input_mut() {
                            last_input.set_video_track(arg);
                        }
                    }
                    "audio-track" | "at" => {
                        if let Some(last_input) = inputs.get_last_input_mut() {
                            last_input.set_audio_track(arg);
                        }
                    }
                    "subtitle-track" | "st" => {
                        if let Some(last_input) = inputs.get_last_input_mut() {
                            last_input.set_subtitle_track(arg);
                        }
                    }
                    "speed" | "spd" => {
                        if let Some(last_input) = inputs.get_last_input_mut() {
                            last_input.set_speed(arg);
                        }
                    }
                    "segment" | "s" => {
                        if let Some(last_input) = inputs.get_last_input_mut() {
                            last_input.add_segment(arg);
                        }
                    }
                    "preset" | "p" => encoder.set_preset(arg),
                    "crf" => encoder.set_crf(arg),
                    "cq" => encoder.set_cq(arg),
                    _ => {}
                }

                current_option = None;
            } else {
                output.set_file(arg);
            }
        }

        Self {
            inputs,
            encoder,
            dry_run,
            output,
        }
    }

    pub fn run(self) {
        if self.dry_run {
            println!("{}", String::from(self));
        } else {
            let _ = Command::new("ffmpeg")
                .args(self)
                .spawn()
                .and_then(|child| child.wait_with_output());
        }
    }

    fn print_help() {
        println!(
            r#"A simple ffmpeg wrapper for clipping videos.

Usage: clipper -input <INPUT> -segment <DURATION RANGE> [OPTIONS] <OUTPUT>

Arguments:
<OUTPUT>  The output file

Options:
-input, -i <INPUT>             Add an input file. This option can be repeated to add more inputs
-video-track, -vt <INDEX>      Set the last input's video track
-audio-track, -at <INDEX>      Set the last input's audio track
-subtitle-track, -st <INDEX>   Burn the last input's subtitle track for all its segments
-speed, -spd <SPEED>           Set the speed multiplier for the last input's segments
-segment, -s <DURATION RANGE>  Add a segment duration range to the last input (e.g. "-segment 2:00-2:30"). This option can be repeated to add more segments
-fade, -f[=<FADE>]             Add a fade transition between all segments. If set (e.g. "-fade=1"), this would be the fade duration in seconds (default: 0.5)
-nvenc                         Encode with NVENC instead of CPU
-hevc                          Convert to HEVC/H.265 instead of AVC/H.264
-preset, -p <PRESET>           Set the encoder preset
-crf <CRF>                     Set the CRF for CPU encoder
-cq <CQ>                       Set the CQ for NVENC encoder
-no-video, -vn                 Disable the video track for all inputs
-no-audio, -an                 Disable the audio track for all inputs
-dry-run, -d                   Output the ffmpeg command instead of directly running ffmpeg
-help, -h                      Print help
-version, -v                   Print version"#
        );

        exit(0);
    }

    fn print_version() {
        println!("v{}", env!("CARGO_PKG_VERSION"));
        exit(0);
    }
}

impl Default for Clipper {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for Clipper {
    type Item = String;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut args = vec![];
        args.extend(self.inputs);
        args.extend(self.encoder);
        args.extend(self.output);
        args.into_iter()
    }
}

impl From<Clipper> for String {
    fn from(value: Clipper) -> Self {
        value.into_iter().fold("ffmpeg".into(), |acc, mut cur| {
            if cur.contains(' ') {
                cur = format!(r#""{cur}""#);
            }
            format!("{acc} {cur}")
        })
    }
}

#[macro_export]
macro_rules! string_vec {
    ($($item:expr),*$(,)?) => (vec![$($item.to_string()),*]);
}

#[macro_export]
macro_rules! error {
    ($string:expr) => {{
        println!("\x1b[38;5;203m{}\x1b[0m", format!($string));
        std::process::exit(1);
    }};
}
