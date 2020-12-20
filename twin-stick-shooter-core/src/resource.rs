use crate::Vec2;

#[derive(Clone, Debug)]
pub struct Time {
    pub elapsed_seconds: f32,
}

#[derive(Clone, Debug)]
pub struct Input {
    pub move_: Vec2,
    pub aim: Vec2,
    pub fire: bool,
}
