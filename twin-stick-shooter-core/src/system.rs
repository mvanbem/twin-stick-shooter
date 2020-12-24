use cgmath::num_traits::zero;
use cgmath::{vec2, InnerSpace, VectorSpace};
use collision::dbvt::{DiscreteVisitor, DynamicBoundingVolumeTree, TreeValueWrapped};
use collision::ComputeBound;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::{Entity, EntityStore, IntoQuery};
use rand_pcg::Pcg32;
use std::collections::HashMap;

use crate::collision::Aabb;
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

#[legion::system]
#[read_component(Position)]
#[write_component(Hitbox)]
#[write_component(HitboxState)]
#[write_component(Hurtbox)]
#[write_component(HurtboxState)]
pub fn collide(
    #[state] dbvt: &mut DynamicBoundingVolumeTree<TreeValueWrapped<Entity, Aabb>>,
    world: &mut SubWorld,
    #[resource] rng: &mut Pcg32,
) {
    const COLLISION_MARGIN: Vec2 = vec2(25.0, 25.0);
    let mut hitbox_entities_by_dbvt_index = HashMap::new();

    // Update all hitboxes.
    for (entity, (hitbox, hitbox_state)) in <(&mut Hitbox, &mut HitboxState)>::query()
        .iter_chunks_mut(world)
        .flat_map(|chunk| chunk.into_iter_entities())
    {
        let value = TreeValueWrapped::new(entity, hitbox.shape.compute_bound(), COLLISION_MARGIN);
        match hitbox.dbvt_index {
            Some(index) => dbvt.update_node(index, value),
            None => hitbox.dbvt_index = Some(dbvt.insert(value)),
        }
        hitbox_entities_by_dbvt_index.insert(hitbox.dbvt_index.unwrap(), entity);
        hitbox_state.hit_entities.clear();
    }

    // Update all hurtboxes.
    for (_entity, (_hurtbox, hurtbox_state)) in <(&mut Hurtbox, &mut HurtboxState)>::query()
        .iter_chunks_mut(world)
        .flat_map(|chunk| chunk.into_iter_entities())
    {
        hurtbox_state.hit_by_entities.clear();
    }

    // Remove any unreferenced DBVT nodes.
    // TODO: Figure out a way to gather these eagerly into a per-frame list?
    let remove_indices: Vec<usize> = dbvt
        .values()
        .iter()
        .filter_map(|&(index, _)| {
            if hitbox_entities_by_dbvt_index.contains_key(&index) {
                None
            } else {
                Some(index)
            }
        })
        .collect();
    for index in remove_indices.into_iter() {
        dbvt.remove(index);
    }

    // Process all collision pairs.
    let mut hurtbox_query = <(&Hurtbox, &mut HurtboxState)>::query();
    let (mut hurtbox_world, mut world) = world.split_for_query(&hurtbox_query);
    let (position_world, mut hitbox_world) = world.split::<&Position>();
    dbvt.tick_with_rng(rng);
    for (hurtbox_entity, (hurtbox, hurtbox_state)) in hurtbox_query
        .iter_chunks_mut(&mut hurtbox_world)
        .flat_map(|chunk| chunk.into_iter_entities())
    {
        let bound = hurtbox.shape.compute_bound();
        let &Position(hurtbox_pos) = position_world
            .entry_ref(hurtbox_entity)
            .unwrap()
            .into_component()
            .unwrap();
        for (value_index, _) in dbvt
            .query_for_indices(
                &mut DiscreteVisitor::<Aabb, TreeValueWrapped<Entity, Aabb>>::new(&bound),
            )
        {
            let node_index = dbvt.values()[value_index].0;
            let hitbox_entity = *hitbox_entities_by_dbvt_index.get(&node_index).unwrap();
            let mut hitbox_entry = hitbox_world.entry_mut(hitbox_entity).unwrap();
            let hitbox: &Hitbox = hitbox_entry.get_component().unwrap();

            if hurtbox.mask.overlaps(hitbox.mask) {
                let &Position(hitbox_pos) = position_world
                    .entry_ref(hitbox_entity)
                    .unwrap()
                    .into_component()
                    .unwrap();
                if crate::collision::test(&hurtbox.shape, hurtbox_pos, &hitbox.shape, hitbox_pos) {
                    hurtbox_state.hit_by_entities.push(hitbox_entity);
                    let hitbox_state: &mut HitboxState = hitbox_entry.get_component_mut().unwrap();
                    hitbox_state.hit_entities.push(hurtbox_entity);
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
