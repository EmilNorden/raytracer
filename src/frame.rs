pub struct Frame {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

impl Frame {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            pixels: vec![0; (width * height * 3) as usize],
            width,
            height,
        }
    }
    
    pub fn get_pixels(&self) -> &[u8] {
        &self.pixels
    }
    
    pub fn width(&self) -> u32 {
        self.width
    }
    
    pub fn height(&self) -> u32 {
        self.height
    }
}