use crate::{
    error,
    ffmpeg::{duration_to_secs, escape_ffmpeg_chars, Input},
    string_vec,
};
use std::vec::IntoIter;

#[derive(Default)]
pub struct Inputs {
    inputs: Vec<Input>,
    fade: Option<f64>,
    no_video: bool,
    no_audio: bool,
}

impl Inputs {
    pub fn add_input(&mut self, file: String) {
        self.inputs.push(Input::new(file));
    }

    pub fn get_last_input_mut(&mut self) -> Option<&mut Input> {
        self.inputs.last_mut()
    }

    pub fn set_fade(&mut self, fade: String) {
        self.fade = fade
            .split('=')
            .last()
            .map(|fade| fade.parse::<f64>().unwrap_or(0.5));
    }

    pub fn set_no_video(&mut self, no_video: bool) {
        self.no_video = no_video;
    }

    pub fn set_no_audio(&mut self, no_audio: bool) {
        self.no_audio = no_audio;
    }
}

impl IntoIterator for Inputs {
    type Item = String;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        if self.inputs.is_empty() {
            error!("Please specify at least one input.");
        }

        if let Some(input) = self.inputs.iter().find(|input| input.segments.is_empty()) {
            error!(format!(r#"Input "{}" has no segments."#, input.file));
        }

        if self.no_video && self.no_audio {
            error!("Video and audio track cannot be disabled at the same time.");
        }

        let mut args = vec![];
        let mut filters = vec![];
        let mut segment_count = 0;

        for (input_index, input) in self.inputs.iter().enumerate() {
            args.append(&mut string_vec!["-i", input.file]);

            let video_label = format!("{input_index}:v:{}", input.video_track);
            let subtitled_video_label = input.subtitle_track.as_ref().map(|subtitle_track| {
                let label = format!("{video_label}:si={subtitle_track}");
                filters.push(format!(
                    "[{video_label}]subtitles={}:si={subtitle_track}[{label}];[{label}]split={}{}",
                    escape_ffmpeg_chars(&input.file),
                    input.segments.len(),
                    (0..input.segments.len())
                        .fold("".into(), |acc, cur| format!("{acc}[{label}:{cur}]")),
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
                        "[{}]trim={from}:{to}",
                        subtitled_video_label.as_ref().map_or_else(
                            || video_label.clone(),
                            |label| format!("{label}:{segment_index}"),
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
                    let mut audio_filters = vec![format!(
                        "[{input_index}:a:{}]atrim={from}:{to}",
                        input.audio_track,
                    )];
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
                "{}concat=n={}:v=1:a=0[v]",
                (0..segment_count).fold("".into(), |acc, cur| format!("{acc}[v{cur}]")),
                segment_count,
            ));
        } else {
            filters.push(format!(
                "{}concat=n={}:v=1:a=1[v][a]",
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

        args.into_iter()
    }
}
