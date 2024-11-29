use crate::{
    ffmpeg::{escape_ffmpeg_chars, Input},
    ffprobe::get_input_metadata,
    string_vec,
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Inputs {
    pub entries: Vec<Input>,
    pub fade: f64,
    pub resize: Option<(u64, u64)>,
    pub no_video: bool,
    pub no_audio: bool,
}

impl Inputs {
    pub fn add_input(&mut self, file: String) {
        self.entries.push(Input::new(file));
    }

    pub fn get_last_input_mut(&mut self) -> Option<&mut Input> {
        self.entries.last_mut()
    }

    pub fn set_fade(&mut self, fade: String) {
        self.fade = fade
            .split('=')
            .last()
            .map(|fade| fade.parse::<f64>().unwrap_or(0.5))
            .unwrap_or(0.);
    }

    pub fn set_resize(&mut self, resolution: String) -> Result<()> {
        let split = resolution
            .split_once([':', 'x'])
            .context(format!("Invalid resize resolution: {resolution}"))?;

        let (mut width, mut height) = (
            split
                .0
                .parse::<u64>()
                .context(format!("Invalid resize width: {}", split.0))?,
            split
                .1
                .parse::<u64>()
                .context(format!("Invalid resize height: {}", split.1))?,
        );

        if width % 2 != 0 {
            width -= 1;
        }

        if height % 2 != 0 {
            height -= 1;
        }

        self.resize = Some((width, height));

        Ok(())
    }

    pub fn set_no_video(&mut self, no_video: bool) {
        self.no_video = no_video;
    }

    pub fn set_no_audio(&mut self, no_audio: bool) {
        self.no_audio = no_audio;
    }

    pub fn try_into_vec(self) -> Result<Vec<String>> {
        if self.entries.is_empty() {
            bail!("Please specify at least one input.");
        }

        if let Some(input) = self.entries.iter().find(|input| input.segments.is_empty()) {
            bail!(r#"Input "{}" has no segments."#, input.file);
        }

        if self.no_video && self.no_audio {
            bail!("Video and audio track cannot be disabled at the same time.");
        }

        let mut input_metadata = vec![];
        let (mut resize_width, mut resize_height) = self.resize.unwrap_or((0, 0));

        for input in self.entries.iter() {
            let metadata = get_input_metadata(input)?;

            if let Some((width, height)) = metadata.resolution {
                if self.resize.is_none() && width > resize_width {
                    resize_width = width;
                    resize_height = height;
                }
            }

            input_metadata.push(metadata);
        }

        // Set the default resize resolution if none of the inputs had a video stream
        if resize_width == 0 || resize_height == 0 {
            resize_width = 1920;
            resize_height = 1080;
        }

        let mut args = vec![];
        let mut filters = vec![];
        let mut segment_count = 0;

        for (input_index, input) in self.entries.iter().enumerate() {
            for (from, to) in &input.segments {
                args.append(&mut string_vec!["-i", input.file]);

                let mut video_label = format!("{segment_count}:v:{}", input.video_track);
                let audio_label = format!("{segment_count}:a:{}", input.audio_track);

                if let Some(subtitle_track) = input.subtitle_track {
                    let new_video_label = format!("{video_label}:si={subtitle_track}");

                    filters.push(format!(
                        "[{video_label}]subtitles={}:si={subtitle_track}[{new_video_label}]",
                        escape_ffmpeg_chars(&input.file),
                    ));

                    video_label = new_video_label;
                }

                let metadata = &input_metadata[input_index];
                let fade = self.fade * input.speed;
                let fade_to = to - fade - 0.5;

                if !self.no_video {
                    if metadata.resolution.is_none() {
                        filters.push(format!(
                            "color=c=black:s={resize_width}x{resize_height}[{video_label}]",
                        ));
                    }

                    let mut video_filters = vec![format!("[{video_label}]trim={from}:{to}")];

                    if self.fade > 0. {
                        video_filters.extend_from_slice(&[
                            format!("fade=t=in:st={from}:d={fade}"),
                            format!("fade=t=out:st={fade_to}:d={fade}"),
                        ]);
                    }

                    if let Some((width, height)) = metadata.resolution {
                        if width != resize_width || height != resize_height {
                            video_filters.extend_from_slice(&[
                                format!("scale={resize_width}:{resize_height}:force_original_aspect_ratio=decrease"),
                                format!("pad={resize_width}:{resize_height}:-1:-1,setsar=1"),
                            ]);
                        }
                    }

                    video_filters.push(format!("setpts=(PTS-STARTPTS)/{}", input.speed));

                    filters.push(format!("{}[v{segment_count}]", video_filters.join(",")));
                }

                if !self.no_audio {
                    if metadata.no_audio {
                        filters.push(format!("anullsrc[{audio_label}]"));
                    }

                    let mut audio_filters = vec![format!("[{audio_label}]atrim={from}:{to}")];

                    if self.fade > 0. {
                        audio_filters.extend_from_slice(&[
                            format!("afade=t=in:st={from}:d={fade}"),
                            format!("afade=t=out:st={fade_to}:d={fade}"),
                        ]);
                    }

                    if input.speed != 1. {
                        audio_filters.push(format!("atempo={}", input.speed));
                    }

                    audio_filters.push("asetpts=PTS-STARTPTS".into());

                    filters.push(format!("{}[a{segment_count}]", audio_filters.join(",")));
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
