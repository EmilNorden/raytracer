use std::sync::atomic::AtomicU64;


pub struct Statistics {
    ray_triangle_tests: AtomicU64,
    ray_triangle_hits: AtomicU64,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            ray_triangle_tests: AtomicU64::new(0),
            ray_triangle_hits: AtomicU64::new(0),
        }
    }

    pub fn ray_triangle_tests(&self) {
        #[cfg(feature = "statistics")]
        {
            self.ray_triangle_tests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub fn ray_triangle_hits(&self) {
        #[cfg(feature = "statistics")]
        {
            self.ray_triangle_hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub fn print_summary(&self) {
        #[cfg(feature = "statistics")]
        {
            println!("Statistics:");
            println!("  Ray triangle tests: {}", self.ray_triangle_tests.load(std::sync::atomic::Ordering::Relaxed));
            println!("  Ray triangle hits: {}", self.ray_triangle_hits.load(std::sync::atomic::Ordering::Relaxed));
            println!("  Ray triangle ratio: {:.2}", self.ray_triangle_hits.load(std::sync::atomic::Ordering::Relaxed) as f32 / self.ray_triangle_tests.load(std::sync::atomic::Ordering::Relaxed) as f32);
        }

        #[cfg(not(feature = "statistics"))]
        {
            println!("Statistics are disabled. To enable them, recompile with the 'statistics' feature.");
        }
    }
}