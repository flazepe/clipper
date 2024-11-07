use crate::string_vec;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
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

    pub fn set_crf(&mut self, crf: String) -> Result<()> {
        if self.nvenc {
            bail!("CRF is only available for CPU encoder.");
        }

        self.crf = Some(
            crf.parse::<f64>()
                .context(format!("Invalid CRF value: {crf}"))?,
        );

        Ok(())
    }

    pub fn set_cq(&mut self, cq: String) -> Result<()> {
        if !self.nvenc {
            bail!("CQ is only available for NVENC encoder.");
        }

        self.cq = Some(
            cq.parse::<f64>()
                .context(format!("Invalid CQ value: {cq}"))?,
        );

        Ok(())
    }

    pub fn try_into_vec(self) -> Result<Vec<String>> {
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

        Ok(args)
    }
}
