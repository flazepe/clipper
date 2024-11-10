use crate::{
    ffmpeg::{escape_ffmpeg_chars, Input},
    string_vec,
};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Inputs {
    pub inputs: Vec<Input>,
    pub fade: f64,
    pub no_video: bool,
    pub no_audio: bool,
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
            .map(|fade| fade.parse::<f64>().unwrap_or(0.5))
            .unwrap_or(0.);
    }

    pub fn set_no_video(&mut self, no_video: bool) {
        self.no_video = no_video;
    }

    pub fn set_no_audio(&mut self, no_audio: bool) {
        self.no_audio = no_audio;
    }

    pub fn try_into_vec(mut self) -> Result<Vec<String>> {
        if self.inputs.is_empty() {
            bail!("Please specify at least one input.");
        }

        if let Some(input) = self.inputs.iter().find(|input| input.segments.is_empty()) {
            bail!(r#"Input "{}" has no segments."#, input.file);
        }

        if self.no_video && self.no_audio {
            bail!("Video and audio track cannot be disabled at the same time.");
        }

        let mut args = vec![];
        let mut filters = vec![];
        let mut segment_count = 0;

        for (input_index, input) in self.inputs.iter().enumerate() {
            args.append(&mut string_vec!["-i", input.file]);

            let video_label = format!("{input_index}:v:{}", input.video_track);
            let label_subtitled_video = input.subtitle_track.as_ref().map(|subtitle_track| {
                let label = format!("{video_label}:si={subtitle_track}");
                filters.push(format!(
                    "[{video_label}]subtitles={}:si={subtitle_track}[{label}];[{label}]split={}{}",
                    escape_ffmpeg_chars(&input.file),
                    input.segments.len(),
                    (0..input.segments.len())
                        .fold("".into(), |acc, cur| format!("{acc}[{label}:{cur}]")),
                ));
                move |segment_index| format!("{label}:{segment_index}")
            });

            for (segment_index, (from, to)) in input.segments.iter().enumerate() {
                let fade_to = to - self.fade * input.speed - 0.5;

                if !self.no_video {
                    let mut video_filters = vec![format!(
                        "[{}]trim={from}:{to}",
                        label_subtitled_video
                            .as_ref()
                            .map_or_else(|| video_label.clone(), |func| func(segment_index)),
                    )];
                    if self.fade > 0. {
                        self.fade *= input.speed;
                        video_filters.extend_from_slice(&[
                            format!("fade=t=in:st={from}:d={}", self.fade),
                            format!("fade=t=out:st={fade_to}:d={}", self.fade),
                        ]);
                    }
                    video_filters.push(format!(
                        "setpts=(PTS-STARTPTS)/{}[v{segment_count}]",
                        input.speed,
                    ));
                    filters.push(video_filters.join(","));
                }

                if !self.no_audio {
                    let mut audio_filters = vec![format!(
                        "[{input_index}:a:{}]atrim={from}:{to}",
                        input.audio_track,
                    )];
                    if self.fade > 0. {
                        audio_filters.extend_from_slice(&[
                            format!("afade=t=in:st={from}:d={}", self.fade),
                            format!("afade=t=out:st={fade_to}:d={}", self.fade),
                        ]);
                    }
                    if input.speed != 1. {
                        audio_filters.push(format!("atempo={}", input.speed));
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

        Ok(args)
    }
}
