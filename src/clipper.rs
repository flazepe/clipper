use crate::ffmpeg::{Encoder, Inputs, Output};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{
    env::args,
    process::{exit, Command},
};

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Clipper {
    pub inputs: Inputs,
    pub encoder: Encoder,
    pub output: Output,
}

impl Clipper {
    pub fn from_env_args() -> Result<Self> {
        let mut clipper = Self::default();
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
                    "fade" | "f" => clipper.inputs.set_fade(arg),
                    "resize" | "r" => current_option = Some(option.into()),
                    "no-video" | "vn" => clipper.inputs.set_no_video(true),
                    "no-audio" | "an" => clipper.inputs.set_no_audio(true),
                    "nvenc" => clipper.encoder.set_nvenc(true),
                    "hevc" => clipper.encoder.set_hevc(true),
                    "preset" | "p" => current_option = Some(option.into()),
                    "crf" => current_option = Some(option.into()),
                    "cq" => current_option = Some(option.into()),
                    "force-overwrite" | "y" => clipper.output.set_force_overwrite(true),
                    "force-not-overwrite" | "n" => clipper.output.set_force_not_overwrite(true),
                    "dry-run" | "d" => clipper.output.set_dry_run(true),
                    "help" | "h" => Self::print_help(),
                    "version" | "v" => Self::print_version(),
                    _ => bail!(
                        r#"Invalid option: -{option}. Use "clipper -help" for more information."#,
                    ),
                }

                continue;
            }

            if let Some(option) = &current_option {
                match option.as_str() {
                    "input" | "i" => clipper.inputs.add_input(arg),
                    "video-track" | "vt" => {
                        if let Some(last_input) = clipper.inputs.get_last_input_mut() {
                            last_input.set_video_track(arg)?;
                        }
                    }
                    "audio-track" | "at" => {
                        if let Some(last_input) = clipper.inputs.get_last_input_mut() {
                            last_input.set_audio_track(arg)?;
                        }
                    }
                    "subtitle-track" | "st" => {
                        if let Some(last_input) = clipper.inputs.get_last_input_mut() {
                            last_input.set_subtitle_track(arg)?;
                        }
                    }
                    "speed" | "spd" => {
                        if let Some(last_input) = clipper.inputs.get_last_input_mut() {
                            last_input.set_speed(arg)?;
                        }
                    }
                    "segment" | "s" => {
                        if let Some(last_input) = clipper.inputs.get_last_input_mut() {
                            last_input.add_segment(arg)?;
                        }
                    }
                    "resize" | "r" => clipper.inputs.set_resize(arg)?,
                    "preset" | "p" => clipper.encoder.set_preset(arg),
                    "crf" => clipper.encoder.set_crf(arg)?,
                    "cq" => clipper.encoder.set_cq(arg)?,
                    _ => {}
                }

                current_option = None;
            } else {
                clipper.output.set_file(arg);
            }
        }

        Ok(clipper)
    }

    pub fn try_into_vec(self) -> Result<Vec<String>> {
        let mut args = vec![];

        args.extend(self.inputs.try_into_vec()?);
        args.extend(self.encoder.try_into_vec()?);
        args.extend(self.output.try_into_vec()?);

        Ok(args)
    }

    pub fn try_into_string(self) -> Result<String> {
        Ok(self
            .try_into_vec()?
            .into_iter()
            .fold("ffmpeg".into(), |acc, mut cur| {
                if cur.contains(' ') {
                    cur = format!(r#""{cur}""#);
                }

                format!("{acc} {cur}")
            }))
    }

    pub fn run(self) -> Result<()> {
        if self.output.dry_run {
            println!("{}", self.try_into_string()?);
        } else {
            let _ = Command::new("ffmpeg")
                .args(self.try_into_vec()?)
                .spawn()
                .and_then(|child| child.wait_with_output());
        }

        Ok(())
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
-resize, -r <RESOLUTION>       Resize all inputs to a specific resolution
-nvenc                         Encode with NVENC instead of CPU
-hevc                          Convert to HEVC/H.265 instead of AVC/H.264
-preset, -p <PRESET>           Set the encoder preset
-crf <CRF>                     Set the CRF for CPU encoder
-cq <CQ>                       Set the CQ for NVENC encoder
-no-video, -vn                 Disable the video track for all inputs
-no-audio, -an                 Disable the audio track for all inputs
-force-overwrite, -y           Force ffmpeg to overwrite the output file without confirmation
-force-not-overwrite, -n       Force ffmpeg to not overwrite the output file without confirmation
-dry-run, -d                   Output the ffmpeg command instead of directly running ffmpeg
-help, -h                      Print help
-version, -v                   Print version"#,
        );

        exit(0);
    }

    fn print_version() {
        println!("v{}", env!("CARGO_PKG_VERSION"));
        exit(0);
    }
}

#[macro_export]
macro_rules! string_vec {
    ($($item:expr),*$(,)?) => (vec![$($item.to_string()),*]);
}
