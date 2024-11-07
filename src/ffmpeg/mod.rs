pub mod encoder;
pub mod input;
pub mod inputs;
pub mod output;

use anyhow::{Context, Result};
pub use encoder::Encoder;
pub use input::Input;
pub use inputs::Inputs;
pub use output::Output;
use std::fmt::Display;

pub fn duration_to_secs<T: Display>(duration: T) -> Result<f64> {
    let mut split = vec![];

    for entry in duration.to_string().split(':') {
        split.push(
            entry
                .parse::<f64>()
                .context(format!("Invalid segment duration: {duration}"))?,
        );
    }

    match split.len() {
        1 => Ok(split[0]),
        2 => Ok((split[0] * 60.) + split[1]),
        3 => Ok((split[0] * 3600.) + (split[1] * 60.) + split[2]),
        _ => Ok(0.),
    }
}

pub fn escape_ffmpeg_chars<T: Display>(string: T) -> String {
    let mut chars = vec![];

    for char in string.to_string().chars() {
        match char {
            '[' | ']' => chars.extend_from_slice(&['\\', char]),
            ':' => chars.extend_from_slice(&['\\', '\\', char]),
            '\'' => chars.extend_from_slice(&['\\', '\\', '\\', char, '\\', '\\', '\\']),
            '\\' => chars.push('/'),
            _ => chars.push(char),
        }
    }

    chars.into_iter().collect()
}
