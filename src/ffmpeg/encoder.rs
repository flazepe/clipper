use crate::string_vec;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Encoder {
    pub nvenc: bool,
    pub hevc: bool,
    pub preset: Option<String>,
    pub crf: Option<f64>,
    pub cq: Option<f64>,
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
        let crf = crf
            .parse::<f64>()
            .context(format!("Invalid CRF value: {crf}"))?;

        if crf > 51. {
            bail!("CRF value must be between 0 and 51. Received: {crf}");
        }

        self.crf = Some(crf);

        Ok(())
    }

    pub fn set_cq(&mut self, cq: String) -> Result<()> {
        let cq = cq
            .parse::<f64>()
            .context(format!("Invalid CQ value: {cq}"))?;

        if cq > 51. {
            bail!("CQ value must be between 0 and 51. Received: {cq}");
        }

        self.cq = Some(cq);

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
