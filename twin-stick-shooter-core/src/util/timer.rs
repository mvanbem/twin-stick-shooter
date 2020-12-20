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

    pub fn step_and_is_elapsed(&mut self, time: &Time) -> bool {
        self.remaining = (self.remaining - time.elapsed_seconds).max(0.0);
        if self.remaining <= 0.0 {
            self.remaining = 0.0;
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self, seconds: f32) {
        self.remaining = seconds;
    }
}
