use crate::denoise::DenoiseFilter;
use crate::frame::Frame;
use crate::options::DenoiseSettings;

#[cfg(feature = "open_image_denoise")]
pub struct Oidn;
impl Oidn {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(feature = "open_image_denoise")]
impl DenoiseFilter for Oidn {
    fn denoise(&self, frame: &Frame, albedo: Option<Frame>, normal: Option<Frame>) -> Frame {
        let mut new_frame = Frame::new(frame.width(), frame.height());

        fn slice_from_frame(frame: &Frame) -> &[f32] {
            unsafe {
                std::slice::from_raw_parts(
                    frame.pixels().as_ptr() as *const f32,
                    frame.pixels().len() * 3
                )
            }
        }
        let input: &[f32] = slice_from_frame(frame);

        let output: &mut [f32] = unsafe {
            std::slice::from_raw_parts_mut(
                new_frame.pixels_mut().as_mut_ptr() as *mut f32,
                frame.pixels().len() * 3
            )
        };

        let device = oidn::device::Device::new();
        let mut rt = oidn::RayTracing::new(&device);
        rt.clean_aux(true);
        rt.srgb(false);
        rt.image_dimensions(frame.width() as usize, frame.height() as usize);
        if let Some(albedo) = albedo {
            let albedo_slice: &[f32] = slice_from_frame(&albedo);
            if let Some(normal) = normal {
                let normal_slice: &[f32] = slice_from_frame(&normal);

                rt.albedo_normal(albedo_slice, normal_slice);
            } else {
                rt.albedo(albedo_slice);
            }
        }

        rt.filter(input, output).expect("Filter config error!");

        if let Err(e) = device.get_error() {
            eprintln!("Error denoising image: {}", e.1);
        }

        new_frame
    }

    fn supports_auxiliary_albedo(&self) -> bool { true }

    fn supports_auxiliary_normal(&self) -> bool { true }
}