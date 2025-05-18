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
    
    pub fn width(&self) -> u32 {
        self.width
    }
    
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn add_sample(&mut self, x: usize, y: usize, sample: Vector3<f32>) {
        self.pixels[x + y * self.width as usize] += sample;
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) {
        let subpixels: Vec<u8> = self.pixels.iter().map(|p| [(p.x * 255.0) as u8, (p.y * 255.0) as u8, (p.z * 255.0) as u8]).flatten().collect();

        let image = image::RgbImage::from_vec(self.width, self.height,subpixels).expect("Failed to create image");
        image.save(path).expect("Failed to save image");
    }
}