use std::collections::HashMap;

use cgmath::vec2;
use collision::dbvt::{DiscreteVisitor, DynamicBoundingVolumeTree, TreeValueWrapped};
use collision::ComputeBound;
use legion::world::SubWorld;
use legion::{Entity, EntityStore, IntoQuery};
use rand_pcg::Pcg32;

use crate::collision::{Aabb, Shape};
use crate::position::Position;
use crate::resource::CollideCounters;
use crate::Vec2;

/// A collider that deals damage.
#[derive(Clone, Debug)]
pub struct Hitbox {
    pub shape: Shape,
    pub dbvt_index: Option<usize>,
    pub mask: HitboxMask,
    pub damage: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct HitboxMask(u32);

impl HitboxMask {
    pub const TARGET: HitboxMask = HitboxMask(0x00000001);

    pub fn overlaps(self, rhs: HitboxMask) -> bool {
        (self.0 & rhs.0) != 0
    }
}

#[derive(Clone, Debug, Default)]
pub struct HitboxState {
    pub hit_entities: Vec<Entity>,
}

/// A collider that receives damage.
#[derive(Clone, Debug)]
pub struct Hurtbox {
    pub shape: Shape,
    pub dbvt_index: Option<usize>,
    pub mask: HitboxMask,
}

#[derive(Clone, Debug, Default)]
pub struct HurtboxState {
    pub hit_by_entities: Vec<Entity>,
}

fn compute_bound(shape: &Shape, pos: Vec2) -> Aabb {
    let local = shape.compute_bound();
    Aabb {
        min: local.min + pos,
        max: local.max + pos,
    }
}

#[legion::system]
#[read_component(Position)]
#[write_component(Hitbox)]
#[write_component(HitboxState)]
#[write_component(Hurtbox)]
#[write_component(HurtboxState)]
pub fn hitbox(
    #[state] dbvt: &mut DynamicBoundingVolumeTree<TreeValueWrapped<Entity, Aabb>>,
    world: &mut SubWorld,
    #[resource] rng: &mut Pcg32,
    #[resource] counters: &mut CollideCounters,
) {
    const COLLISION_MARGIN: Vec2 = vec2(25.0, 25.0);
    let mut hitbox_entities_by_dbvt_index = HashMap::new();
    *counters = CollideCounters::default();

    // Update all hitboxes.
    for (entity, (&Position(pos), hitbox, hitbox_state)) in
        <(&Position, &mut Hitbox, &mut HitboxState)>::query()
            .iter_chunks_mut(world)
            .flat_map(|chunk| chunk.into_iter_entities())
    {
        counters.hitboxes += 1;
        let value =
            TreeValueWrapped::new(entity, compute_bound(&hitbox.shape, pos), COLLISION_MARGIN);
        match hitbox.dbvt_index {
            Some(index) => {
                counters.dbvt_updates += 1;
                dbvt.update_node(index, value);
            }
            None => {
                counters.dbvt_inserts += 1;
                hitbox.dbvt_index = Some(dbvt.insert(value));
            }
        }
        hitbox_entities_by_dbvt_index.insert(hitbox.dbvt_index.unwrap(), entity);
        hitbox_state.hit_entities.clear();
    }

    // Update all hurtboxes.
    for hurtbox_state in <&mut HurtboxState>::query().iter_mut(world) {
        counters.hurtboxes += 1;
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
        counters.dbvt_removes += 1;
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
        let &Position(hurtbox_pos) = position_world
            .entry_ref(hurtbox_entity)
            .unwrap()
            .into_component()
            .unwrap();
        let bound = compute_bound(&hurtbox.shape, hurtbox_pos);

        counters.dbvt_queries += 1;
        for (value_index, ()) in dbvt
            .query_for_indices(
                &mut DiscreteVisitor::<Aabb, TreeValueWrapped<Entity, Aabb>>::new(&bound),
            )
        {
            counters.dbvt_hits += 1;

            let node_index = dbvt.values()[value_index].0;
            let hitbox_entity = *hitbox_entities_by_dbvt_index.get(&node_index).unwrap();
            let mut hitbox_entry = hitbox_world.entry_mut(hitbox_entity).unwrap();
            let hitbox: &Hitbox = hitbox_entry.get_component().unwrap();

            if hurtbox.mask.overlaps(hitbox.mask) {
                counters.mask_hits += 1;

                let &Position(hitbox_pos) = position_world
                    .entry_ref(hitbox_entity)
                    .unwrap()
                    .into_component()
                    .unwrap();
                if crate::collision::test(&hurtbox.shape, hurtbox_pos, &hitbox.shape, hitbox_pos) {
                    counters.gjk_hits += 1;

                    hurtbox_state.hit_by_entities.push(hitbox_entity);
                    let hitbox_state: &mut HitboxState = hitbox_entry.get_component_mut().unwrap();
                    hitbox_state.hit_entities.push(hurtbox_entity);
                } else {
                    counters.gjk_misses += 1;
                }
            } else {
                counters.mask_misses += 1;
            }
        }
    }
}
