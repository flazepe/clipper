use crate::ffmpeg::duration_to_secs;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub file: String,
    pub segments: Vec<(f64, f64)>,
    pub video_track: u8,
    pub audio_track: u8,
    pub subtitle_track: Option<u8>,
    pub speed: f64,
}

impl Input {
    pub fn new(file: String) -> Self {
        Self {
            file: file.to_string(),
            segments: vec![],
            video_track: 0,
            audio_track: 0,
            subtitle_track: None,
            speed: 1.,
        }
    }

    pub fn add_segment(&mut self, segment: String) -> Result<()> {
        let split = segment
            .split_once('-')
            .context(format!("Invalid segment duration range: {segment}"))?;

        let (from, to) = (duration_to_secs(split.0)?, duration_to_secs(split.1)?);
        self.segments.push((from, to));

        Ok(())
    }

    pub fn set_video_track(&mut self, video_track: String) -> Result<()> {
        self.video_track = video_track
            .parse::<u8>()
            .context(format!("Invalid video track: {video_track}"))?;

        Ok(())
    }

    pub fn set_audio_track(&mut self, audio_track: String) -> Result<()> {
        self.audio_track = audio_track
            .parse::<u8>()
            .context(format!("Invalid audio track: {audio_track}"))?;

        Ok(())
    }

    pub fn set_subtitle_track(&mut self, subtitle_track: String) -> Result<()> {
        self.subtitle_track = Some(
            subtitle_track
                .parse::<u8>()
                .context(format!("Invalid subtitle track: {subtitle_track}"))?,
        );

        Ok(())
    }

    pub fn set_speed(&mut self, speed: String) -> Result<()> {
        let speed = speed
            .parse::<f64>()
            .context(format!("Invalid speed multiplier: {speed}"))?;

        if !(0.5..100.).contains(&speed) {
            bail!("Speed multiplier must be between 0.5 and 100. Received: {speed}");
        }

        self.speed = speed;

        Ok(())
    }
}
