use crate::Vec2;

#[derive(Clone, Debug)]
pub struct Time {
    pub elapsed_seconds: f32,
}

/// Interpolation factor in the closed-open interval [0, 1), with zero at the previous position and
/// one at the current position.
#[derive(Clone, Debug)]
pub struct Subframe(pub f32);

#[derive(Clone, Debug)]
pub struct Input {
    pub move_: Vec2,
    pub aim: Vec2,
    pub fire: bool,
}
