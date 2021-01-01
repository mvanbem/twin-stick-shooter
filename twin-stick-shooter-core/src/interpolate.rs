use cgmath::{EuclideanSpace, VectorSpace};

use crate::position::PositionComponent;
use crate::resource::Subframe;
use crate::{Pt2, Vec2};

#[derive(Clone, Debug)]
pub struct InterpolateComponent {
    pub prev_pos: Pt2,
    pub interpolated_pos: Pt2,
}

#[legion::system(for_each)]
pub fn interpolate(
    &PositionComponent(pos): &PositionComponent,
    &mut InterpolateComponent {
        prev_pos,
        ref mut interpolated_pos,
    }: &mut InterpolateComponent,
    #[resource] &Subframe(subframe): &Subframe,
) {
    *interpolated_pos = Pt2::from_vec(Vec2::lerp(prev_pos.to_vec(), pos.to_vec(), subframe));
}
