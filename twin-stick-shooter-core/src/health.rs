use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::{Entity, EntityStore};

use crate::hitbox::{HitboxComponent, HitboxEffect, HurtboxComponent};
use crate::resource::Time;
use crate::util::Timer;

#[derive(Clone, Debug)]
pub struct HealthComponent {
    pub health: f32,
    pub hit_flash: Timer,
}

impl HealthComponent {
    pub fn new(health: f32) -> HealthComponent {
        HealthComponent {
            health,
            hit_flash: Timer::elapsed(),
        }
    }

    pub fn is_hit_flashing(&self) -> bool {
        !self.hit_flash.is_elapsed()
    }
}

const HIT_FLASH_DURATION_SECONDS: f32 = 0.05;

#[legion::system(for_each)]
#[read_component(HitboxComponent)]
pub fn damage(
    cmd: &mut CommandBuffer,
    world: &SubWorld,
    entity: &Entity,
    hurtbox: &HurtboxComponent,
    health: &mut HealthComponent,
    #[resource] time: &Time,
) {
    health.hit_flash.step(time);

    // Take damage from all colliding hitboxes.
    for hitbox_entity in &hurtbox.hit_by_entities {
        let hitbox: &HitboxComponent = world
            .entry_ref(*hitbox_entity)
            .unwrap()
            .into_component()
            .unwrap();
        if let HitboxEffect::Damage(damage) = hitbox.effect {
            health.health = (health.health - damage).max(0.0);
            health.hit_flash.reset(HIT_FLASH_DURATION_SECONDS);
        }
    }

    if health.health == 0.0 {
        cmd.remove(*entity);
    }
}
