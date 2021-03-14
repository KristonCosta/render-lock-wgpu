use std::time::Instant;

#[derive(Debug)]
pub struct TimeStep {
    pub last_time: Instant,
    delta_time: f64,
    frame_count: u32,
    frame_time: f64,
    pub frame_rate: u64,
}

impl TimeStep {
    // https://gitlab.com/flukejones/diir-doom/blob/master/game/src/main.rs
    // Grabbed this from here
    pub fn new() -> TimeStep {
        TimeStep {
            last_time: Instant::now(),
            delta_time: 0.0,
            frame_count: 0,
            frame_time: 0.0,
            frame_rate: 0,
        }
    }

    pub fn delta(&mut self) -> f64 {
        let current_time = Instant::now();
        let delta = current_time.duration_since(self.last_time).as_millis() as f64;
        self.last_time = current_time;
        self.delta_time = delta;
        self.frame_count += 1;
        self.frame_time += self.delta_time;

        // per second
        if self.frame_time >= 1000.0 {
            self.frame_rate = self.frame_count as u64;
            self.frame_count = 0;
            self.frame_time = 0.0;
        }
        delta
    }
}
