use crate::denoise::DenoiseFilter;
use crate::frame::Frame;

pub struct Passthrough;

impl Passthrough {
    pub fn new() -> Self {
        Self {}
    }
}

impl DenoiseFilter for Passthrough {
    fn denoise(&self, frame: &Frame, _albedo: Option<Frame>, _normal: Option<Frame>) -> Frame {
        let mut new_frame = Frame::new(frame.width(), frame.height());
        new_frame.pixels_mut().copy_from_slice(frame.pixels());

        new_frame
    }

    fn supports_auxiliary_albedo(&self) -> bool { false }
    fn supports_auxiliary_normal(&self) -> bool { false }
}