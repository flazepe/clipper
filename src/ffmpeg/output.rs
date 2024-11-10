use crate::string_vec;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub file: Option<String>,
    pub force_overwrite: bool,
    pub force_not_overwrite: bool,
    pub dry_run: bool,
}

impl Output {
    pub fn set_file(&mut self, file: String) {
        self.file = Some(file);
    }

    pub fn set_force_overwrite(&mut self, force_overwrite: bool) {
        self.force_overwrite = force_overwrite;
    }

    pub fn set_force_not_overwrite(&mut self, force_not_overwrite: bool) {
        self.force_not_overwrite = force_not_overwrite;
    }

    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    pub fn try_into_vec(self) -> Result<Vec<String>> {
        let mut args = string_vec!["-pix_fmt", "yuv420p"];

        args.push(self.file.context("Please specify an output file.")?);

        if self.force_overwrite {
            args.push("-y".into());
        } else if self.force_not_overwrite {
            args.push("-n".into());
        }

        Ok(args)
    }
}
