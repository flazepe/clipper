use crate::{
    error,
    ffmpeg::{duration_to_secs, escape_ffmpeg_chars},
    string_vec,
};

pub struct Inputs {
    pub inputs: Vec<Input>,
    pub fade: Option<f64>,
    pub no_video: bool,
    pub no_audio: bool,
}

impl Inputs {
    pub fn to_args(&self) -> Vec<String> {
        let mut args = vec![];
        let mut filters = vec![];
        let mut segment_count = 0;

        for (input_index, input) in self.inputs.iter().enumerate() {
            args.append(&mut string_vec!["-i", input.file]);

            let video_label = format!("{input_index}:v:{}", input.video_track.unwrap_or(0));
            let subtitled_video_label = input.subtitle_track.as_ref().map(|subtitle_track| {
                let label = format!("{video_label}s{subtitle_track}");
                filters.push(format!(
                    r#"[{video_label}]subtitles={}:si={subtitle_track}[{label}];[{label}]split={}{}"#,
                    escape_ffmpeg_chars(&input.file),
                    input.segments.len(),
                    (0..input.segments.len())
                        .fold("".into(), |acc, cur| format!("{acc}[{label}p{cur}]")),
                ));
                label
            });

            for (segment_index, segment) in input.segments.iter().enumerate() {
                let (from, to) = segment
                    .split_once('-')
                    .map(|(from, to)| (duration_to_secs(from), duration_to_secs(to)))
                    .unwrap_or_else(|| {
                        error!(format!("Invalid segment duration range: {segment}"));
                    });
                let fade_to = to - self.fade.unwrap_or(0.) - 0.5;

                if !self.no_video {
                    let mut video_filters = vec![format!(
                        "[{}]trim={from}{to}",
                        subtitled_video_label.as_ref().map_or_else(
                            || video_label.clone(),
                            |label| format!("{label}p{segment_index}"),
                        ),
                    )];
                    if let Some(fade) = self.fade {
                        video_filters.extend_from_slice(&[
                            format!("fade=t=in:st={from}:d={fade}"),
                            format!("fade=t=out:st={fade_to}:d={fade}"),
                        ]);
                    }
                    video_filters.push(format!("setpts=PTS-STARTPTS[v{segment_count}]"));
                    filters.push(video_filters.join(","));
                }

                if !self.no_audio {
                    let audio_track = input.audio_track.unwrap_or(0);
                    let mut audio_filters =
                        vec![format!("[{input_index}:a:{audio_track}]atrim={from}{to}")];
                    if let Some(fade) = self.fade {
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

        if self.no_video {
            filters.push(format!(
                "{}concat=n={}:v=0:a=1[a]",
                (0..segment_count).fold("".into(), |acc, cur| format!("{acc}[a{cur}]")),
                segment_count,
            ));
        } else if self.no_audio {
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

        args.append(&mut string_vec!["-filter_complex", filters.join(";")]);

        if !self.no_video {
            args.append(&mut string_vec!["-map", "[v]"]);
        }

        if !self.no_audio {
            args.append(&mut string_vec!["-map", "[a]"]);
        }

        args
    }
}

pub struct Input {
    pub file: String,
    pub segments: Vec<String>,
    pub video_track: Option<u8>,
    pub audio_track: Option<u8>,
    pub subtitle_track: Option<u8>,
}
