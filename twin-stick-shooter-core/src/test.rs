use cgmath::InnerSpace;

use crate::physics::Velocity;
use crate::position::Position;

#[derive(Clone, Debug)]
pub struct ReflectWithin(pub f32);

#[legion::system(for_each)]
pub fn reflect_within(
    &Position(pos): &Position,
    Velocity(vel): &mut Velocity,
    &ReflectWithin(reflect_within): &ReflectWithin,
) {
    // Check if the entity is outside the reflecting circle.
    if pos.magnitude() >= reflect_within {
        // Check if the entity's velocity is outward.
        let radial = vel.dot(pos.normalize());
        if radial > 0.0 {
            // Reflect the entity's velocity inward by subtracting the radial component twice. Note
            // that subtracting once would merely put its motion perpendicular to the reflecting
            // circle.
            *vel -= pos.normalize_to(2.0 * radial);
        }
    }
}
