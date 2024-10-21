use crate::args::Args;
use std::{fmt::Display, process::exit, process::Command};

pub fn spawn_ffmpeg(args: Args) {
    let mut ffmpeg_args = vec!["-i", &args.input];
    let mut filters = vec![];

    // Segment ranges
    if args.segments.iter().any(|segment| segment.contains('-')) {
        for (index, segment) in args.segments.iter().enumerate() {
            let (from, to) = segment
                .split_once('-')
                .map(|(from, to)| (duration_to_secs(from), duration_to_secs(to)))
                .unwrap_or_else(|| {
                    print_error("A non-range value found inside a segment!");
                    exit(1);
                });

            let fade_to = to - (args.fade.unwrap_or(0.) + 0.5);

            // Video filters
            let mut video_filters = vec![format!("[0:v]trim={from}:{to}")];
            if let Some(fade) = args.fade {
                video_filters.extend_from_slice(&[
                    format!("fade=t=in:st={from}:d={fade}"),
                    format!("fade=t=out:st={fade_to}:d={fade}"),
                ]);
            }
            video_filters.push(format!("setpts=PTS-STARTPTS[v{index}]"));

            // Audio filters
            let mut audio_filters = vec![format!("[0:a]atrim={from}:{to}")];
            if let Some(fade) = args.fade {
                audio_filters.extend_from_slice(&[
                    format!("afade=t=in:st={from}:d={fade}"),
                    format!("afade=t=out:st={fade_to}:d={fade}"),
                ]);
            }
            audio_filters.push(format!("asetpts=PTS-STARTPTS[a{index}]"));

            // Push filters
            filters.extend_from_slice(&[video_filters.join(","), audio_filters.join(",")]);
        }

        filters.push(format!(
            "{}concat=n={}:v=1:a=1[v][a]",
            (0..args.segments.len())
                .map(|index| format!("[v{index}][a{index}]"))
                .collect::<Vec<String>>()
                .join(""),
            args.segments.len(),
        ));
    }

    // Join filters
    let filters = filters.join(";");

    if filters.is_empty() {
        ffmpeg_args.extend_from_slice(&["-ss", &args.segments[0]]);
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

fn duration_to_secs<T: Display>(duration: T) -> f64 {
    let split = duration
        .to_string()
        .split(':')
        .map(|entry| {
            entry.parse::<f64>().unwrap_or_else(|_| {
                print_error("Invalid segment duration found inside a range!");
                exit(1);
            })
        })
        .collect::<Vec<f64>>();

    match split.len() {
        1 => split[0],
        2 => (split[0] * 60.) + split[1],
        3 => (split[0] * 3600.) + (split[1] * 60.) + split[2],
        _ => 0.,
    }
}

fn print_error(message: &str) {
    println!("\x1b[38;5;203m{message}\x1b[0m")
}
