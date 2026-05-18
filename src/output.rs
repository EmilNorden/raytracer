use std::io::Write;
use std::time::Instant;
use humantime::format_duration;

pub struct TaskOutput {
    pub start: Instant,
}

impl TaskOutput {
    pub fn new(msg: &str) -> TaskOutput {
        print!("{}... ", msg);
        std::io::stdout().flush().unwrap();

        TaskOutput { start: Instant::now() }
    }

    pub fn done(self) {
        println!("Done ({})", format_duration(self.start.elapsed()));
    }
}