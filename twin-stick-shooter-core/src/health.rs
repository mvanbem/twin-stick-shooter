use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::{Entity, EntityStore};

use crate::hitbox::{Hitbox, HurtboxState};

#[derive(Clone, Debug)]
pub struct Health(pub f32);

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
