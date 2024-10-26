use crate::error;

pub struct Input {
    pub file: String,
    pub segments: Vec<String>,
    pub video_track: u8,
    pub audio_track: u8,
    pub subtitle_track: Option<u8>,
}

impl Input {
    pub fn new(file: String) -> Self {
        Self {
            file: file.to_string(),
            segments: vec![],
            video_track: 0,
            audio_track: 0,
            subtitle_track: None,
        }
    }

    pub fn add_segment(&mut self, segment: String) {
        self.segments.push(segment);
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
}
