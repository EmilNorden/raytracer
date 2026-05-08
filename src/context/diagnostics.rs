use std::sync::atomic::AtomicU64;

pub struct Diagnostics {
    eta_underflows: AtomicU64,
    exit_without_enter: AtomicU64,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            eta_underflows: AtomicU64::new(0),
            exit_without_enter: AtomicU64::new(0),
        }
    }

    #[inline(always)]
    pub fn eta_underflow(&self) {
        #[cfg(feature = "diagnostics")]
        {
            self.eta_underflows.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub fn exit_without_enter(&self) {
        #[cfg(feature = "diagnostics")]
        {
            self.exit_without_enter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub fn print_summary(&self) {
        #[cfg(feature = "diagnostics")]
        {
            println!("Diagnostics:");
            println!("  Eta underflows: {}", self.eta_underflows.load(std::sync::atomic::Ordering::Relaxed));
            println!("  Exit without enter: {}", self.exit_without_enter.load(std::sync::atomic::Ordering::Relaxed));
        }

        #[cfg(not(feature = "diagnostics"))]
        {
            println!("Diagnostics are disabled. To enable them, recompile with the 'diagnostics' feature.");
        }
    }


}