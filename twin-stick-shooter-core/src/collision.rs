use cgmath::{Basis2, Decomposed, One};
use collision::CollisionStrategy;
use std::fmt::Debug;

use crate::Vec2;

pub type Aabb = collision::Aabb2<f32>;
pub type Circle = collision::primitive::Circle<f32>;
pub type Shape = collision::primitive::Primitive2<f32>;

type GJK = collision::algorithm::minkowski::GJK2<f32>;

#[derive(Clone, Copy, Debug)]
pub struct CollisionMask(u32);

impl CollisionMask {
    pub const TARGET: CollisionMask = CollisionMask(0x00000001);

    pub fn overlaps(self, rhs: CollisionMask) -> bool {
        (self.0 & rhs.0) != 0
    }
}

pub fn test(shape_a: &Shape, pos_a: Vec2, shape_b: &Shape, pos_b: Vec2) -> bool {
    let gjk = GJK::new();
    gjk.intersection(
        &CollisionStrategy::CollisionOnly,
        shape_a,
        &Decomposed {
            scale: 1.0,
            rot: Basis2::one(),
            disp: pos_a,
        },
        shape_b,
        &Decomposed {
            scale: 1.0,
            rot: Basis2::one(),
            disp: pos_b,
        },
    )
    .is_some()
}
