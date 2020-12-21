use cgmath::num_traits::zero;
use cgmath::{InnerSpace, VectorSpace};
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::{Entity, EntityStore, IntoQuery};

use crate::collision::Shape;
use crate::component::{
    ForceAccumulator, Health, Hitbox, HitboxState, Hurtbox, HurtboxState, InterpolatedPosition,
    Lifespan, Mass, Position, PrevPosition, ReflectWithin, RemoveOnHit, Velocity,
};
use crate::resource::{Subframe, Time};
use crate::Vec2;

mod player;

pub use player::{player_act_system, player_plan_system};

#[legion::system(for_each)]
pub fn lifespan(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    lifespan: &mut Lifespan,
    #[resource] time: &Time,
) {
    if lifespan.0.step_and_is_elapsed(time) {
        cmd.remove(*entity);
    }
}

#[legion::system(for_each)]
pub fn physics(
    #[resource] time: &Time,
    mass: Option<&Mass>,
    pos: &mut Position,
    prev_pos: Option<&mut PrevPosition>,
    vel: &mut Velocity,
    force: Option<&mut ForceAccumulator>,
) {
    if let (Some(mass), Some(force)) = (mass, force) {
        vel.0 += force.0 * mass.inv_mass() * time.elapsed_seconds;
        force.0 = zero();
    }
    if let Some(prev_pos) = prev_pos {
        prev_pos.0 = pos.0;
    }
    pos.0 += vel.0 * time.elapsed_seconds;
}

#[legion::system]
#[read_component(Position)]
#[read_component(Hitbox)]
#[read_component(Hurtbox)]
#[write_component(HitboxState)]
#[write_component(HurtboxState)]
pub fn collide(world: &mut SubWorld) {
    // Clear all hitbox states.
    for hitbox_state in <&mut HitboxState>::query().iter_mut(world) {
        hitbox_state.hit_entities.clear();
    }

    // Visit all entities with hurtboxes.
    let mut hurtbox_query = <(&Hurtbox, &mut HurtboxState)>::query();
    let (mut hurtbox_world, mut hitbox_world) = world.split_for_query(&hurtbox_query);
    let mut hitbox_query = <(&Hitbox, &mut HitboxState)>::query();
    let (mut hitbox_world, position_world) = hitbox_world.split_for_query(&hitbox_query);

    for (hurtbox_entity, (hurtbox, hurtbox_state)) in hurtbox_query
        .iter_chunks_mut(&mut hurtbox_world)
        .flat_map(|chunk| chunk.into_iter_entities())
    {
        hurtbox_state.hit_by_entities.clear();

        // Check all hitboxes against this hurtbox.
        //
        // TODO: Use some kind of broadphase to make this not n^2 lol
        for (hitbox_entity, (hitbox, hitbox_state)) in hitbox_query
            .iter_chunks_mut(&mut hitbox_world)
            .flat_map(|chunk| chunk.into_iter_entities())
        {
            if hitbox_entity != hurtbox_entity && hitbox.mask.overlaps(hurtbox.mask) {
                let hitbox_pos = position_world
                    .entry_ref(hitbox_entity)
                    .unwrap()
                    .get_component::<Position>()
                    .unwrap()
                    .0;
                let hurtbox_pos = position_world
                    .entry_ref(hurtbox_entity)
                    .unwrap()
                    .get_component::<Position>()
                    .unwrap()
                    .0;
                if Shape::test(&hitbox.shape, hitbox_pos, &hurtbox.shape, hurtbox_pos) {
                    hitbox_state.hit_entities.push(hurtbox_entity);
                    hurtbox_state.hit_by_entities.push(hitbox_entity);
                }
            }
        }
    }
}

#[legion::system(for_each)]
#[read_component(Hitbox)]
pub fn damage(
    cmd: &mut CommandBuffer,
    world: &SubWorld,
    entity: &Entity,
    hurtbox_state: &HurtboxState,
    health: &mut Health,
) {
    let Health(ref mut health) = health;

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
    pos: &Position,
    prev_pos: &PrevPosition,
    interpolated_pos: &mut InterpolatedPosition,
    #[resource] subframe: &Subframe,
) {
    interpolated_pos.0 = Vec2::lerp(prev_pos.0, pos.0, subframe.0);
}

#[legion::system(for_each)]
pub fn reflect_within(pos: &Position, vel: &mut Velocity, reflect_within: &ReflectWithin) {
    // Check if the entity is outside the reflecting circle.
    if pos.0.magnitude() >= reflect_within.0 {
        // Check if the entity's velocity is outward.
        let radial = vel.0.dot(pos.0.normalize());
        if radial > 0.0 {
            // Reflect the entity's velocity inward by subtracting the radial component twice. Note
            // that subtracting once would merely put its motion perpendicular to the reflecting
            // circle.
            vel.0 -= pos.0.normalize_to(2.0 * radial);
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
