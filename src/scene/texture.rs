use nalgebra::Vector3;

pub struct Texture {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

#[derive(Copy, Clone, Debug)]
pub enum Channel {
    R, G, B
}

impl Channel {
    pub fn index(self) -> usize {
        match self {
            Channel::R => 0,
            Channel::G => 1,
            Channel::B => 2,
        }
    }
}

impl Texture {
    pub fn new(pixels: Vec<u8>, width: u32, height: u32) -> Self {
        assert!(width > 0 && height > 0, "texture dimensions must be non-zero");
        assert_eq!(pixels.len(), width as usize * height as usize * 4);
        Self {
            pixels,
            width,
            height,
        }
    }

    pub fn sample_color(&self, u: f32, v: f32) -> Vector3<f32> {
        assert!(u.is_finite(), "u is not finite: {}", u);
        assert!(v.is_finite(), "v is not finite: {}", v);

        // Clamp to avoid edge cases where u or v is exactly 1.0.
        let x = ((u.clamp(0.0, 1.0) * self.width as f32) as usize).min(self.width as usize - 1);
        let y = ((v.clamp(0.0, 1.0) * self.height as f32) as usize).min(self.height as usize - 1);

        let idx = 4 * (x + y * self.width as usize);
        let r = self.pixels[idx];
        let g = self.pixels[idx + 1];
        let b = self.pixels[idx + 2];
        Vector3::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }

    pub fn sample_channel(&self, u: f32, v: f32, channel: Channel) -> f32 {
        assert!(u.is_finite(), "u is not finite: {}", u);
        assert!(v.is_finite(), "v is not finite: {}", v);
        let x = ((u.clamp(0.0, 1.0) * self.width as f32) as usize).min(self.width as usize - 1);
        let y = ((v.clamp(0.0, 1.0) * self.height as f32) as usize).min(self.height as usize - 1);
        let idx = 4 * (x + y * self.width as usize) + channel.index() as usize;
        self.pixels[idx] as f32 / 255.0
    }
}