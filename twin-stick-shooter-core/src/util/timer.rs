use crate::resource::Time;

#[derive(Clone, Debug)]
pub struct Timer {
    remaining: f32,
}

impl Timer {
    pub fn elapsed() -> Timer {
        Timer { remaining: 0.0 }
    }

    pub fn with_remaining(remaining: f32) -> Timer {
        Timer { remaining }
    }

    pub fn step(&mut self, time: &Time) {
        self.remaining = (self.remaining - time.elapsed_seconds).max(0.0);
    }

    pub fn is_elapsed(&self) -> bool {
        self.remaining == 0.0
    }

    pub fn step_and_is_elapsed(&mut self, time: &Time) -> bool {
        self.step(time);
        self.is_elapsed()
    }

    pub fn reset(&mut self, seconds: f32) {
        self.remaining = seconds;
    }

    pub fn elapse_now(&mut self) {
        self.remaining = 0.0;
    }
}
