use std::sync::atomic::AtomicU64;

pub struct Memory {
    texture_memory_bytes: AtomicU64,
    mesh_memory_bytes: AtomicU64,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            texture_memory_bytes: AtomicU64::new(0),
            mesh_memory_bytes: AtomicU64::new(0),
        }
    }

    pub fn texture_memory_bytes(&self, bytes: u64) {
        self.texture_memory_bytes.fetch_add(bytes, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn mesh_memory_bytes(&self, bytes: u64) {
        self.mesh_memory_bytes.fetch_add(bytes, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn print_summary(&self) {
        println!("Memory usage:");
        println!("  Textures: {}", size::Size::from_bytes(self.texture_memory_bytes.load(std::sync::atomic::Ordering::Relaxed)));
        println!("  Meshes: {}", size::Size::from_bytes(self.mesh_memory_bytes.load(std::sync::atomic::Ordering::Relaxed)));
    }
}