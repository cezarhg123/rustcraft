use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Timer {
    prev_time: Instant,
    crnt_time: Instant
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            prev_time: Instant::now(),
            crnt_time: Instant::now()
        }
    }

    pub fn tick(&mut self) {
        self.crnt_time = Instant::now();
    }

    pub fn elapsed(&self) -> f32 {
        (self.crnt_time - self.prev_time).as_secs_f32()
    }

    pub fn elapsed_millis(&self) -> u64 {
        (self.crnt_time - self.prev_time).as_millis() as u64
    }

    pub fn reset(&mut self) {
        self.prev_time = self.crnt_time;
    }
}
