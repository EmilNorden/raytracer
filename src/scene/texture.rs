use nalgebra::Vector3;

pub struct Texture {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

impl Texture {
    pub fn new(pixels: Vec<u8>, width: u32, height: u32) -> Self {
        assert_eq!(pixels.len(), width as usize * height as usize * 4);
        Self {
            pixels,
            width,
            height,
        }
    }

    pub fn sample_color(&self, u: f32, v: f32) -> Vector3<f32> {
        assert!(u >= 0.0 && u <= 1.0);
        assert!(v >= 0.0 && v <= 1.0);
        let x = (u * self.width as f32) as usize;
        let y = (v * self.height as f32) as usize;

        let r = self.pixels[4 * (x + y * self.width as usize)];
        let g = self.pixels[4 * (x + y * self.width as usize) + 1];
        let b = self.pixels[4 * (x + y * self.width as usize) + 2];
        Vector3::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }
}