use crate::string_vec;

pub struct Output(pub String);

impl Output {
    pub fn to_args(&self) -> Vec<String> {
        let mut args = string_vec!["-pix_fmt", "yuv420p"];
        args.push(self.0.clone());
        args
    }
}
