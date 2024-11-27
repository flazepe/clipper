use crate::ffmpeg::Input;
use anyhow::Result;
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{from_str, Value};
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct InputMetadata {
    pub resolution: Option<(u64, u64)>,
    pub no_audio: bool,
}

#[derive(Deserialize, Debug)]
struct InputStreams<T> {
    streams: Vec<T>,
}

#[derive(Deserialize, Debug)]
struct InputVideoStream {
    width: u64,
    height: u64,
}

pub fn get_input_metadata(input: &Input) -> Result<InputMetadata> {
    let mut video_streams = ffprobe::<InputStreams<InputVideoStream>>(
        &input.file,
        &format!("V:{}", input.video_track), // Capital V to ignore covers
        "stream=width,height,sample_aspect_ratio,display_aspect_ratio",
    )?;

    let audio_streams = ffprobe::<InputStreams<Value>>(
        &input.file,
        &format!("a:{}", input.audio_track),
        "stream=index",
    )?;

    Ok(InputMetadata {
        resolution: video_streams.streams.first_mut().map(|stream| {
            if stream.width % 2 != 0 {
                stream.width -= 1;
            }

            if stream.height % 2 != 0 {
                stream.height -= 1;
            }

            (stream.width, stream.height)
        }),
        no_audio: audio_streams.streams.is_empty(),
    })
}

pub fn ffprobe<T: DeserializeOwned>(file: &str, stream: &str, entries: &str) -> Result<T> {
    let output = Command::new("ffprobe")
        .args([
            "-i",
            file,
            "-select_streams",
            stream,
            "-show_entries",
            entries,
            "-of",
            "json",
        ])
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    Ok(from_str::<T>(&String::from_utf8(output.stdout)?)?)
}
