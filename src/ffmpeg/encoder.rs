use crate::string_vec;

pub enum Encoder {
    Cpu {
        hevc: bool,
        preset: Option<String>,
        crf: Option<f64>,
    },
    Nvenc {
        hevc: bool,
        preset: Option<String>,
        cq: Option<f64>,
    },
}

impl Encoder {
    pub fn to_args(&self) -> Vec<String> {
        let mut args = string_vec!["-c:v"];

        match self {
            Self::Cpu { hevc, preset, crf } => {
                if *hevc {
                    args.push("libx265".into());
                } else {
                    args.push("libx264".into());
                }

                if let Some(preset) = preset {
                    args.append(&mut string_vec!["-preset", preset]);
                }

                if let Some(crf) = crf {
                    args.append(&mut string_vec!["-crf", crf]);
                }
            }
            Self::Nvenc { hevc, preset, cq } => {
                if *hevc {
                    args.push("hevc_nvenc".into());
                } else {
                    args.push("h264_nvenc".into());
                }

                if let Some(preset) = preset {
                    args.append(&mut string_vec!["-preset", preset]);
                }

                if let Some(cq) = cq {
                    args.append(&mut string_vec!["-cq", cq]);
                }
            }
        };

        args
    }
}
