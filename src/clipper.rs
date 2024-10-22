use crate::{args::Args, error};
use std::fmt::Display;
use std::process::Command;

pub struct Clipper(Args);

impl Clipper {
    pub fn new(args: Args) -> Self {
        Self(args)
    }

    pub fn run(&self) {
        let ffmpeg_args = self.generate_ffmpeg_args();

        if self.0.dry_run {
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
        ffmpeg_args.append(&mut self.generate_ffmpeg_input_and_filter_args());
        ffmpeg_args.append(&mut self.generate_ffmpeg_encoder_args());
        ffmpeg_args.push(self.0.output.clone());
        ffmpeg_args
    }

    fn generate_ffmpeg_input_and_filter_args(&self) -> Vec<String> {
        let mut args = vec![];
        let mut filters = vec![];
        let mut segment_count = 0;

        for (input_index, (input, segments)) in self.0.input.iter().enumerate() {
            args.extend_from_slice(&["-i".into(), input.into()]);

            for segment in segments {
                let (from, to) = segment
                    .split_once('-')
                    .map(|(from, to)| (Self::duration_to_secs(from), Self::duration_to_secs(to)))
                    .unwrap_or_else(|| {
                        error!(format!("Invalid segment duration range: {segment}"))
                    });
                let fade_to = to - self.0.fade.unwrap_or(0.) - 0.5;

                if !self.0.no_video {
                    let mut video_filters = vec![format!("[{input_index}:v]trim={from}:{to}")];
                    if let Some(fade) = self.0.fade {
                        video_filters.extend_from_slice(&[
                            format!("fade=t=in:st={from}:d={fade}"),
                            format!("fade=t=out:st={fade_to}:d={fade}"),
                        ]);
                    }
                    video_filters.push(format!("setpts=PTS-STARTPTS[v{segment_count}]"));
                    filters.push(video_filters.join(","));
                }

                if !self.0.no_audio {
                    let mut audio_filters = vec![format!("[{input_index}:a]atrim={from}:{to}")];
                    if let Some(fade) = self.0.fade {
                        audio_filters.extend_from_slice(&[
                            format!("afade=t=in:st={from}:d={fade}"),
                            format!("afade=t=out:st={fade_to}:d={fade}"),
                        ]);
                    }
                    audio_filters.push(format!("asetpts=PTS-STARTPTS[a{segment_count}]"));
                    filters.push(audio_filters.join(","));
                }

                segment_count += 1;
            }
        }

        if self.0.no_video {
            filters.push(format!(
                "{}concat=n={}:v=0:a=1[a]",
                (0..segment_count).fold("".into(), |acc, cur| format!("{acc}[a{cur}]")),
                segment_count,
            ));
        } else if self.0.no_audio {
            filters.push(format!(
                "{}concat=n={}[v]",
                (0..segment_count).fold("".into(), |acc, cur| format!("{acc}[v{cur}]")),
                segment_count,
            ));
        } else {
            filters.push(format!(
                "{}concat=n={}:a=1[v][a]",
                (0..segment_count).fold("".into(), |acc, cur| format!("{acc}[v{cur}][a{cur}]")),
                segment_count,
            ));
        }

        args.extend_from_slice(&["-filter_complex".into(), filters.join(";")]);

        if !self.0.no_video {
            args.extend_from_slice(&["-map".into(), "[v]".into()]);
        }

        if !self.0.no_audio {
            args.extend_from_slice(&["-map".into(), "[a]".into()]);
        }

        args
    }

    fn generate_ffmpeg_encoder_args(&self) -> Vec<String> {
        if let Some(cq) = self.0.cq.as_ref() {
            vec![
                "-c:v".into(),
                if self.0.hevc {
                    "hevc_nvenc".into()
                } else {
                    "h264_nvenc".into()
                },
                "-cq".into(),
                cq.into(),
            ]
        } else {
            vec![
                "-c:v".into(),
                if self.0.hevc {
                    "libx265".into()
                } else {
                    "libx264".into()
                },
            ]
        }
    }

    fn duration_to_secs<T: Display>(duration: T) -> f64 {
        let split = duration
            .to_string()
            .split(':')
            .map(|entry| {
                entry
                    .parse::<f64>()
                    .unwrap_or_else(|_| error!(format!("Invalid segment duration: {entry}")))
            })
            .collect::<Vec<f64>>();

        match split.len() {
            1 => split[0],
            2 => (split[0] * 60.) + split[1],
            3 => (split[0] * 3600.) + (split[1] * 60.) + split[2],
            _ => 0.,
        }
    }
}

#[macro_export]
macro_rules! error {
    ($message:expr) => {{
        println!("\x1b[38;5;203m{}\x1b[0m", $message);
        std::process::exit(1);
    }};
}
