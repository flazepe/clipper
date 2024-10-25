use crate::{error, string_vec};
use std::vec::IntoIter;

#[derive(Default)]
pub struct Output(Option<String>);

impl Output {
    pub fn set_file(&mut self, file: String) {
        self.0 = Some(file);
    }
}

impl IntoIterator for Output {
    type Item = String;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut args = string_vec!["-pix_fmt", "yuv420p"];

        match self.0 {
            Some(file) => args.push(file),
            None => error!("Please specify an output file."),
        }

        args.into_iter()
    }
}
