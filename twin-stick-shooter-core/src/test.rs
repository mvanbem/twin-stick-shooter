use cgmath::{EuclideanSpace, InnerSpace};

use crate::physics::VelocityComponent;
use crate::position::PositionComponent;

#[derive(Clone, Debug)]
pub struct ReflectWithin(pub f32);

#[legion::system(for_each)]
pub fn reflect_within(
    &PositionComponent(pos): &PositionComponent,
    VelocityComponent(vel): &mut VelocityComponent,
    &ReflectWithin(reflect_within): &ReflectWithin,
) {
    // Check if the entity is outside the reflecting circle.
    if pos.to_vec().magnitude() >= reflect_within {
        // Check if the entity's velocity is outward.
        let radial = vel.dot(pos.to_vec().normalize());
        if radial > 0.0 {
            // Reflect the entity's velocity inward by subtracting the radial component twice. Note
            // that subtracting once would merely put its motion perpendicular to the reflecting
            // circle.
            *vel -= pos.to_vec().normalize_to(2.0 * radial);
        }
    }
}
