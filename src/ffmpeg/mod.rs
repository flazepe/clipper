pub mod encoder;
pub mod input;
pub mod inputs;
pub mod output;

use crate::error;
pub use encoder::Encoder;
pub use input::Input;
pub use inputs::Inputs;
pub use output::Output;
use std::fmt::Display;

pub fn duration_to_secs<T: Display>(duration: T) -> f64 {
    let split = duration
        .to_string()
        .split(':')
        .map(|entry| {
            entry
                .parse::<f64>()
                .unwrap_or_else(|_| error!("Invalid segment duration: {entry}"))
        })
        .collect::<Vec<f64>>();

    match split.len() {
        1 => split[0],
        2 => (split[0] * 60.) + split[1],
        3 => (split[0] * 3600.) + (split[1] * 60.) + split[2],
        _ => 0.,
    }
}

pub fn escape_ffmpeg_chars<T: Display>(string: T) -> String {
    let mut chars = vec![];

    for char in string.to_string().chars() {
        match char {
            '\'' | '[' | '\\' | ']' => {
                chars.extend_from_slice(&['\\', '\\', '\\', char, '\\', '\\', '\\'])
            }
            ':' => {
                chars.extend_from_slice(&['\\', '\\', char]);
            }
            _ => chars.push(char),
        }
    }

    chars.into_iter().collect()
}
