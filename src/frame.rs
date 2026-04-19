use std::path::Path;
use nalgebra::Vector3;

pub struct Frame {
    pixels: Vec<Vector3<f32>>,
    width: u32,
    height: u32,
}


impl Frame {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            pixels: vec![Vector3::default(); (width * height) as usize],
            width,
            height,
        }
    }

    pub fn clear(&mut self) {
        self.pixels.iter_mut().for_each(|p| *p = Vector3::default());
    }

    pub fn pixels_mut(&mut self) -> &mut [Vector3<f32>] {
        &mut self.pixels
    }
    
    pub fn width(&self) -> u32 {
        self.width
    }
    
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn add_sample(&mut self, x: usize, y: usize, sample: Vector3<f32>) {
        self.pixels[x + y * self.width as usize] += sample;
    }

    pub fn write_rgba(&self, output: &mut [u8]) {
        assert_eq!(output.len(), (self.width * self.height * 4) as usize);

        for (pixel, rgba) in self.pixels.iter().zip(output.chunks_exact_mut(4)) {
            rgba[0] = Self::to_display_u8(pixel.x);
            rgba[1] = Self::to_display_u8(pixel.y);
            rgba[2] = Self::to_display_u8(pixel.z);
            rgba[3] = 255;
        }
    }

    fn to_display_u8(value: f32) -> u8 {
        let gamma_corrected = value.clamp(0.0, 1.0).powf(1.0 / 2.2);
        (gamma_corrected * 255.0).round() as u8
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) {
        let subpixels: Vec<u8> = self
            .pixels
            .iter()
            .flat_map(|p| {
                [
                    (p.x.clamp(0.0, 1.0) * 255.0).round() as u8,
                    (p.y.clamp(0.0, 1.0) * 255.0).round() as u8,
                    (p.z.clamp(0.0, 1.0) * 255.0).round() as u8,
                ]
            })
            .collect();

        let image = image::RgbImage::from_vec(self.width, self.height,subpixels).expect("Failed to create image");
        image.save(path).expect("Failed to save image");
    }
}