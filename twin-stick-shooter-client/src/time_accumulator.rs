use std::ops::Div;

use derive_more::{Add, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Add, SubAssign)]
pub struct Seconds(pub f32);

impl Seconds {
    pub fn seconds(self) -> f32 {
        self.0
    }

    pub fn at_most(self, x: Seconds) -> Seconds {
        Seconds(self.0.min(x.0))
    }
}

impl Div<Seconds> for Seconds {
    type Output = f32;

    fn div(self, rhs: Seconds) -> Self::Output {
        self.0 / rhs.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Sub)]
pub struct Milliseconds(pub f64);

impl Milliseconds {
    fn to_seconds(self) -> Seconds {
        Seconds((self.0 / 1000.0) as f32)
    }
}

#[derive(Clone, Debug, Default)]
pub struct TimeAccumulator {
    last_timestamp: Option<Milliseconds>,
    accumulator: Seconds,
}

impl TimeAccumulator {
    pub fn accumulator(&self) -> Seconds {
        self.accumulator
    }

    pub fn update_for_timestamp(&mut self, timestamp: Milliseconds) {
        if let Some(last_timestamp) = self.last_timestamp {
            let dt = (timestamp - last_timestamp).to_seconds();
            // Allow simulation to slow down if running below 10 fps.
            self.accumulator = (self.accumulator + dt).at_most(Seconds(0.1));
        }
        self.last_timestamp = Some(timestamp);
    }

    pub fn try_consume(&mut self, dt: Seconds) -> bool {
        if self.accumulator > dt {
            self.accumulator -= dt;
            true
        } else {
            false
        }
    }
}
