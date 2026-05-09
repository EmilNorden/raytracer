pub mod diagnostics;
mod statistics;
mod memory;

pub struct Context {
    pub diag: diagnostics::Diagnostics,
    pub stats: statistics::Statistics,
    pub mem: memory::Memory,
}

impl Context {
    pub fn new() -> Self {
        Self {
            diag: diagnostics::Diagnostics::new(),
            stats: statistics::Statistics::new(),
            mem: memory::Memory::new(),
        }
    }

    pub fn finalize(&self) {
        self.diag.print_summary();
        self.stats.print_summary();
    }
}