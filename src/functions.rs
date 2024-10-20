use crate::args::Args;
use std::{fmt::Display, process::Command};

pub fn spawn_ffmpeg(args: Args) {
    let mut ffmpeg_args = vec!["-i", &args.input];
    let mut filters = vec![];

    // Ranges
    if args.segments.contains('-') {
        let segments = args
            .segments
            .split([' ', ',', ';'])
            .map(|segment| segment.to_string())
            .collect::<Vec<String>>();

        for (index, segment) in segments.iter().enumerate() {
            let (from, to) = segment
                .split_once('-')
                .map(|(from, to)| (duration_to_secs(from), duration_to_secs(to)))
                .unwrap_or_else(|| panic!("{}", "Invalid segment range!".error_text()));

            let fade_to = if to < 1. { to } else { to - 1. };

            // Video filters
            let mut video_filters = vec![format!("[0:v]trim={from}:{to}")];
            if args.fade {
                video_filters.extend_from_slice(&[
                    format!("fade=t=in:st={from}:d=1"),
                    format!("fade=t=out:st={fade_to}:d=1"),
                ]);
            }
            video_filters.push(format!("setpts=PTS-STARTPTS[v{index}]"));

            // Audio filters
            let mut audio_filters = vec![format!("[0:a]atrim={from}:{to}")];
            if args.fade {
                audio_filters.extend_from_slice(&[
                    format!("afade=t=in:st={from}:d=1"),
                    format!("afade=t=out:st={fade_to}:d=1"),
                ]);
            }
            audio_filters.push(format!("asetpts=PTS-STARTPTS[a{index}]"));

            // Push filters
            filters.extend_from_slice(&[video_filters.join(","), audio_filters.join(",")]);
        }

        filters.push(format!(
            "{}concat=n={}:v=1:a=1[v][a]",
            (0..segments.len())
                .map(|index| format!("[v{index}][a{index}]"))
                .collect::<Vec<String>>()
                .join(""),
            segments.len(),
        ));
    }

    // Join filters
    let filters = filters.join(";");

    if filters.is_empty() {
        ffmpeg_args.extend_from_slice(&["-ss", &args.segments]);
    } else {
        ffmpeg_args.extend_from_slice(&["-filter_complex", &filters, "-map", "[v]", "-map", "[a]"]);
    }

    // Encoders
    if let Some(cq) = &args.cq {
        ffmpeg_args.extend_from_slice(&[
            "-c:v",
            if args.hevc {
                "hevc_nvenc"
            } else {
                "h264_nvenc"
            },
            "-cq",
            cq,
        ]);
    } else {
        ffmpeg_args.extend_from_slice(&["-c:v", if args.hevc { "libx265" } else { "libx264" }]);
    }

    // Mute
    if args.mute {
        ffmpeg_args.push("-an");
    }

    ffmpeg_args.push(&args.output);

    // Debug
    if args.debug {
        return println!(
            "ffmpeg {}",
            ffmpeg_args
                .iter()
                .map(|arg| if arg.contains(' ') {
                    format!(r#""{arg}""#)
                } else {
                    arg.to_string()
                })
                .collect::<Vec<String>>()
                .join(" ")
        );
    }

    let _ = Command::new("ffmpeg")
        .args(ffmpeg_args)
        .spawn()
        .and_then(|child| child.wait_with_output());
}

pub fn duration_to_secs<T: Display>(duration: T) -> f64 {
    let split = duration
        .to_string()
        .split(':')
        .map(|entry| {
            entry
                .parse::<f64>()
                .unwrap_or_else(|_| panic!("{}", "Invalid segment duration!".error_text()))
        })
        .collect::<Vec<f64>>();

    match split.len() {
        1 => split[0],
        2 => (split[0] * 60.) + split[1],
        3 => (split[0] * 3600.) + (split[1] * 60.) + split[2],
        _ => 0.,
    }
}

trait ErrorText {
    fn error_text(self) -> String;
}

impl<T: Display> ErrorText for T {
    fn error_text(self) -> String {
        format!("\x1b[38;5;203m{self}\x1b[0m")
    }
}
