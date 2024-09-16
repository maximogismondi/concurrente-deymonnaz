use std::time::{Duration, Instant};

/// A simple timer to measure the time taken by a program.
pub struct Timer {
    start: Instant,
    last_lap: Instant,
}

impl Timer {
    /// Creates a new `Timer` instance.
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            last_lap: Instant::now(),
        }
    }

    /// Resets the last lap time to the current time.
    /// Returns the duration since the last lap.
    /// The first lap will be the time since the timer was created.
    pub fn lap(&mut self) -> Duration {
        let now = Instant::now();
        let lap = now - self.last_lap;
        self.last_lap = now;
        lap
    }

    /// Returns the total duration since the timer was created.
    pub fn total(&self) -> Duration {
        Instant::now() - self.start
    }

    /// Prints the duration since the last lap with a message.
    pub fn print_lap(&mut self, message: &str) {
        let lap = self.lap();
        println!("{}: {:.2?}", message, lap);
    }

    /// Prints the total duration since the timer was created.
    pub fn print_total(&self) {
        let total = self.total();
        println!("Total: {:.2?}", total);
    }
}
