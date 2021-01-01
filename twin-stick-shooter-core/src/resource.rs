use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::Vec2;

#[derive(Clone, Debug)]
pub struct Time {
    pub elapsed_seconds: f32,
}

/// Interpolation factor in the closed-open interval [0, 1), with zero at the previous position and
/// one at the current position.
#[derive(Clone, Debug)]
pub struct Subframe(pub f32);

/// TODO: These fields are an incoherent mix of player inputs and standard-mapped gamepad inputs.
/// Pick one.
#[derive(Clone, Debug)]
pub struct Input {
    pub move_: Vec2,
    pub aim: Vec2,
    pub fire: bool,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub confirm: bool,
    pub start: bool,
}

#[derive(Debug, Default)]
pub struct CollideCounters {
    pub hitboxes: usize,
    pub hurtboxes: usize,
    pub dbvt_inserts: usize,
    pub dbvt_updates: usize,
    pub dbvt_removes: usize,
    pub dbvt_queries: usize,
    pub dbvt_hits: usize,
    pub mask_hits: usize,
    pub mask_misses: usize,
    pub gjk_hits: usize,
    pub gjk_misses: usize,
}

#[derive(Clone, Debug)]
pub enum GuiOverride {
    StationDocked,
}

#[derive(Clone, Debug, Default)]
pub struct GuiOverrideQueue {
    queue: Arc<Mutex<VecDeque<GuiOverride>>>,
}

impl GuiOverrideQueue {
    pub fn push_back(&self, gui_override: GuiOverride) {
        self.queue.lock().unwrap().push_back(gui_override);
    }

    pub fn drain(&self) -> Vec<GuiOverride> {
        self.queue.lock().unwrap().drain(..).collect()
    }
}
