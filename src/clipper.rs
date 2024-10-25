use crate::{
    error,
    ffmpeg::{Encoder, Inputs, Output},
};
use std::{env::args, process::Command, vec::IntoIter};

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
                    _ => error!(format!("Invalid option: -{option}")),
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
        value.into_iter().fold("ffmpeg".into(), |acc, cur| {
            format!(
                "{acc} {}",
                if cur.contains(' ') {
                    format!(r#""{cur}""#)
                } else {
                    cur.to_string()
                },
            )
        })
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
