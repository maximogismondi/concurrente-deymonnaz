use std::time::{Duration, Instant};

pub struct Timer {
    start: Instant,
    last_lap: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            last_lap: Instant::now(),
        }
    }

    pub fn lap(&mut self) -> Duration {
        let now = Instant::now();
        let lap = now - self.last_lap;
        self.last_lap = now;
        lap
    }

    pub fn total(&self) -> Duration {
        Instant::now() - self.start
    }

    pub fn print_lap(&mut self, message: &str) {
        let lap = self.lap();
        println!("{}: {:.2?}", message, lap);
    }

    pub fn print_total(&self) {
        let total = self.total();
        println!("Total: {:.2?}", total);
    }
}
