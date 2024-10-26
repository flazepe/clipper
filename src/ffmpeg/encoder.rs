use crate::{error, string_vec};
use std::vec::IntoIter;

#[derive(Default)]
pub struct Encoder {
    nvenc: bool,
    hevc: bool,
    preset: Option<String>,
    crf: Option<f64>,
    cq: Option<f64>,
}

impl Encoder {
    pub fn set_nvenc(&mut self, nvenc: bool) {
        self.nvenc = nvenc;
    }

    pub fn set_hevc(&mut self, hevc: bool) {
        self.hevc = hevc;
    }

    pub fn set_preset(&mut self, preset: String) {
        self.preset = Some(preset);
    }

    pub fn set_crf(&mut self, crf: String) {
        if self.nvenc {
            error!("CRF is only available for CPU encoder.");
        }

        self.crf = Some(crf.parse::<f64>().unwrap_or_else(|_| {
            error!("Invalid CRF value: {crf}");
        }));
    }

    pub fn set_cq(&mut self, cq: String) {
        if !self.nvenc {
            error!("CQ is only available for NVENC encoder.");
        }

        self.cq = Some(cq.parse::<f64>().unwrap_or_else(|_| {
            error!("Invalid CQ value: {cq}");
        }));
    }
}

impl IntoIterator for Encoder {
    type Item = String;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut args = string_vec!["-c:v"];

        if self.nvenc {
            if self.hevc {
                args.push("hevc_nvenc".into());
            } else {
                args.push("h264_nvenc".into());
            }

            if let Some(cq) = self.cq {
                args.append(&mut string_vec!["-cq", cq]);
            }
        } else {
            if self.hevc {
                args.push("libx265".into());
            } else {
                args.push("libx264".into());
            }

            if let Some(crf) = self.crf {
                args.append(&mut string_vec!["-crf", crf]);
            }
        }

        if let Some(preset) = &self.preset {
            args.append(&mut string_vec!["-preset", preset]);
        }

        args.into_iter()
    }
}
