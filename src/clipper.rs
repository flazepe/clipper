use crate::{
    error,
    ffmpeg::{Encoder, Input, Inputs, Output},
};
use std::{env::args, process::Command};

pub struct Clipper {
    pub inputs: Inputs,
    pub encoder: Encoder,
    pub output: Output,
    pub dry_run: bool,
}

impl Clipper {
    pub fn new() -> Self {
        let mut inputs = Inputs {
            inputs: vec![],
            fade: None,
            no_video: false,
            no_audio: false,
        };
        let mut nvenc = false;
        let mut hevc = false;
        let mut preset = None;
        let mut crf = None;
        let mut cq = None;
        let mut output = None;
        let mut dry_run = false;
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
                    "segment" | "s" => current_option = Some(option.into()),
                    "fade" | "f" => {
                        inputs.fade = arg
                            .split('=')
                            .last()
                            .map(|fade| fade.parse::<f64>().unwrap_or(0.5));
                    }
                    "nvenc" => nvenc = true,
                    "hevc" => hevc = true,
                    "preset" | "p" => current_option = Some(option.into()),
                    "crf" => current_option = Some(option.into()),
                    "cq" => current_option = Some(option.into()),
                    "no-video" | "vn" => inputs.no_video = true,
                    "no-audio" | "an" => inputs.no_audio = true,
                    "dry-run" | "d" => dry_run = true,
                    _ => error!(format!("Invalid option: -{option}")),
                }

                continue;
            }

            if let Some(option) = &current_option {
                match option.as_str() {
                    "input" | "i" => inputs.inputs.push(Input {
                        file: arg,
                        segments: vec![],
                        video_track: None,
                        audio_track: None,
                        subtitle_track: None,
                    }),
                    "video-track" | "vt" => {
                        if let Some(last_input) = inputs.inputs.last_mut() {
                            last_input.video_track = Some(arg.parse::<u8>().unwrap_or_else(|_| {
                                error!(format!("Invalid video track: {arg}"));
                            }));
                        }
                    }
                    "audio-track" | "at" => {
                        if let Some(last_input) = inputs.inputs.last_mut() {
                            last_input.audio_track = Some(arg.parse::<u8>().unwrap_or_else(|_| {
                                error!(format!("Invalid audio track: {arg}"));
                            }));
                        }
                    }
                    "subtitle-track" | "st" => {
                        if let Some(last_input) = inputs.inputs.last_mut() {
                            last_input.subtitle_track =
                                Some(arg.parse::<u8>().unwrap_or_else(|_| {
                                    error!(format!("Invalid subtitle track: {arg}"));
                                }));
                        }
                    }
                    "segment" | "s" => {
                        if let Some(last_input) = inputs.inputs.last_mut() {
                            last_input.segments.push(arg);
                        }
                    }
                    "preset" | "p" => preset = Some(arg),
                    "crf" => {
                        crf = Some(
                            arg.parse::<f64>()
                                .unwrap_or_else(|_| error!(format!("Invalid CRF value: {arg}"))),
                        );
                    }
                    "cq" => {
                        cq = Some(
                            arg.parse::<f64>()
                                .unwrap_or_else(|_| error!(format!("Invalid CQ value: {arg}"))),
                        );
                    }
                    _ => {}
                }

                current_option = None;
            } else {
                output = Some(Output(arg));
            }
        }

        // Validations
        if inputs.inputs.is_empty() {
            error!("Please enter at least one input.");
        }

        if let Some(input) = inputs.inputs.iter().find(|input| input.segments.is_empty()) {
            error!(format!(r#"Input "{}" has no segments."#, input.file));
        }

        if inputs.no_video && inputs.no_audio {
            error!("Video and audio track cannot be disabled at the same time.");
        }

        let Some(output) = output else {
            error!("Please specify an output file.");
        };

        Self {
            inputs,
            encoder: if nvenc {
                Encoder::Nvenc { hevc, preset, cq }
            } else {
                Encoder::Cpu { hevc, preset, crf }
            },
            dry_run,
            output,
        }
    }

    pub fn run(&self) {
        let ffmpeg_args = self.generate_ffmpeg_args();

        if self.dry_run {
            println!(
                "{}",
                ffmpeg_args.iter().fold("ffmpeg".into(), |acc, cur| format!(
                    "{acc} {}",
                    if cur.contains(' ') {
                        format!(r#""{cur}""#)
                    } else {
                        cur.to_string()
                    },
                )),
            );
        } else {
            let _ = Command::new("ffmpeg")
                .args(ffmpeg_args)
                .spawn()
                .and_then(|child| child.wait_with_output());
        }
    }

    fn generate_ffmpeg_args(&self) -> Vec<String> {
        let mut ffmpeg_args = vec![];
        ffmpeg_args.append(&mut self.inputs.to_args());
        ffmpeg_args.append(&mut self.encoder.to_args());
        ffmpeg_args.append(&mut self.output.to_args());
        ffmpeg_args
    }
}

#[macro_export]
macro_rules! string_vec {
    ($($item:expr),*$(,)?) => (vec![$($item.to_string()),*]);
}

#[macro_export]
macro_rules! error {
    ($message:expr) => {{
        println!("\x1b[38;5;203m{}\x1b[0m", $message);
        std::process::exit(1);
    }};
}
