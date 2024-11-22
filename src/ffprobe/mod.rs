use crate::ffmpeg::Input;
use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{from_str, Value};
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct InputMetadata {
    pub width: u64,
    pub height: u64,
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
    let video_metadata = run_ffprobe::<InputStreams<InputVideoStream>>(
        &input.file,
        &format!("v:{}", input.video_track),
        "stream=width,height",
    )?;

    let (mut width, mut height) = video_metadata
        .streams
        .first()
        .map(|stream: &InputVideoStream| (stream.width, stream.height))
        .context(format!(
            r#"Input "{}" has no video track {}."#,
            input.file, input.video_track,
        ))?;

    if width % 2 != 0 {
        width -= 1;
    }

    if height % 2 != 0 {
        height -= 1;
    }

    let audio_metadata = run_ffprobe::<InputStreams<Value>>(
        &input.file,
        &format!("a:{}", input.audio_track),
        "stream=index",
    )?;

    Ok(InputMetadata {
        width,
        height,
        no_audio: audio_metadata.streams.is_empty(),
    })
}

pub fn run_ffprobe<T: DeserializeOwned>(file: &str, stream: &str, entries: &str) -> Result<T> {
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
