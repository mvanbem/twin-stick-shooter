use collision::CollisionStrategy;

use crate::Mat3;

pub type Aabb = collision::Aabb2<f32>;
pub type Circle = collision::primitive::Circle<f32>;
pub type Shape = collision::primitive::Primitive2<f32>;

type GJK = collision::algorithm::minkowski::GJK2<f32>;

pub fn test(shape_a: &Shape, xform_a: &Mat3, shape_b: &Shape, xform_b: &Mat3) -> bool {
    let gjk = GJK::new();
    gjk.intersection(
        &CollisionStrategy::CollisionOnly,
        shape_a,
        xform_a,
        shape_b,
        xform_b,
    )
    .is_some()
}
