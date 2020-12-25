use cgmath::num_traits::zero;
use cgmath::{InnerSpace, VectorSpace};
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::{Entity, EntityStore};

use crate::component::{
    ForceAccumulator, Health, Hitbox, HitboxState, HurtboxState, InterpolatedPosition, Lifespan,
    Mass, Position, PrevPosition, ReflectWithin, RemoveOnHit, Velocity,
};
use crate::resource::{Subframe, Time};
use crate::Vec2;

pub mod collide;
pub mod player;

#[legion::system(for_each)]
pub fn lifespan(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    Lifespan(lifespan): &mut Lifespan,
    #[resource] time: &Time,
) {
    if lifespan.step_and_is_elapsed(time) {
        cmd.remove(*entity);
    }
}

#[legion::system(for_each)]
pub fn physics(
    #[resource] time: &Time,
    mass: Option<&Mass>,
    Position(pos): &mut Position,
    prev_pos: Option<&mut PrevPosition>,
    Velocity(vel): &mut Velocity,
    force: Option<&mut ForceAccumulator>,
) {
    if let (Some(mass), Some(ForceAccumulator(force))) = (mass, force) {
        *vel += *force * mass.inv_mass() * time.elapsed_seconds;
        *force = zero();
    }
    if let Some(PrevPosition(prev_pos)) = prev_pos {
        *prev_pos = *pos;
    }
    *pos += *vel * time.elapsed_seconds;
}

#[legion::system(for_each)]
#[read_component(Hitbox)]
pub fn damage(
    cmd: &mut CommandBuffer,
    world: &SubWorld,
    entity: &Entity,
    hurtbox_state: &HurtboxState,
    Health(health): &mut Health,
) {
    // Take damage from all colliding hitboxes.
    for hitbox_entity in &hurtbox_state.hit_by_entities {
        let hitbox: &Hitbox = world
            .entry_ref(*hitbox_entity)
            .unwrap()
            .into_component()
            .unwrap();
        *health = (*health - hitbox.damage).max(0.0);
    }

    if *health == 0.0 {
        cmd.remove(*entity);
    }
}

#[legion::system(for_each)]
pub fn interpolate(
    &Position(pos): &Position,
    &PrevPosition(prev_pos): &PrevPosition,
    InterpolatedPosition(interpolated_pos): &mut InterpolatedPosition,
    #[resource] &Subframe(subframe): &Subframe,
) {
    *interpolated_pos = Vec2::lerp(prev_pos, pos, subframe);
}

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

#[legion::system(for_each)]
pub fn remove_on_hit(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    hitbox_state: &HitboxState,
    _: &RemoveOnHit,
) {
    if !hitbox_state.hit_entities.is_empty() {
        cmd.remove(*entity);
    }
}
