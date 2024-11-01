use crate::{error, ffmpeg::duration_to_secs};

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

    pub fn add_segment(&mut self, segment: String) {
        let (from, to) = segment
            .split_once('-')
            .map(|(from, to)| (duration_to_secs(from), duration_to_secs(to)))
            .unwrap_or_else(|| {
                error!("Invalid segment duration range: {segment}");
            });

        self.segments.push((from, to));
    }

    pub fn set_video_track(&mut self, video_track: String) {
        self.video_track = video_track.parse::<u8>().unwrap_or_else(|_| {
            error!("Invalid video track: {video_track}");
        });
    }

    pub fn set_audio_track(&mut self, audio_track: String) {
        self.audio_track = audio_track.parse::<u8>().unwrap_or_else(|_| {
            error!("Invalid audio track: {audio_track}");
        });
    }

    pub fn set_subtitle_track(&mut self, subtitle_track: String) {
        self.subtitle_track = Some(subtitle_track.parse::<u8>().unwrap_or_else(|_| {
            error!("Invalid subtitle track: {subtitle_track}");
        }));
    }

    pub fn set_speed(&mut self, speed: String) {
        let speed = speed.parse::<f64>().unwrap_or_else(|_| {
            error!("Invalid speed multiplier: {speed}");
        });

        if !(0.5..100.).contains(&speed) {
            error!("Speed multiplier must be between 0.5 and 100. Received: {speed}");
        }

        self.speed = speed;
    }
}
