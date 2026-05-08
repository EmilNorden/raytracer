pub mod diagnostics;
mod statistics;

pub struct Context {
    pub diag: diagnostics::Diagnostics,
    pub stats: statistics::Statistics,
}

impl Context {
    pub fn new() -> Self {
        Self {
            diag: diagnostics::Diagnostics::new(),
            stats: statistics::Statistics::new(),
        }
    }

    pub fn finalize(&self) {
        self.diag.print_summary();
        self.stats.print_summary();
    }
}