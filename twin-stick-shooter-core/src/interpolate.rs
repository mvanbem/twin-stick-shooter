use cgmath::VectorSpace;

use crate::position::Position;
use crate::resource::Subframe;
use crate::Vec2;

#[derive(Clone, Debug)]
pub struct Interpolate {
    pub prev_pos: Vec2,
    pub interpolated_pos: Vec2,
}

#[legion::system(for_each)]
pub fn interpolate(
    &Position(pos): &Position,
    Interpolate {
        prev_pos,
        interpolated_pos,
    }: &mut Interpolate,
    #[resource] &Subframe(subframe): &Subframe,
) {
    *interpolated_pos = Vec2::lerp(*prev_pos, pos, subframe);
}
