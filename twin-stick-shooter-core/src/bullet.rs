use legion::systems::CommandBuffer;
use legion::Entity;

use crate::hitbox::HitboxState;
use crate::resource::Time;
use crate::util::Timer;

#[derive(Clone, Debug)]
pub struct LifespanComponent(pub Timer);

#[derive(Clone, Debug)]
pub struct RemoveOnHitComponent;

#[legion::system(for_each)]
pub fn lifespan(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    LifespanComponent(lifespan): &mut LifespanComponent,
    #[resource] time: &Time,
) {
    if lifespan.step_and_is_elapsed(time) {
        cmd.remove(*entity);
    }
}

#[legion::system(for_each)]
pub fn remove_on_hit(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    hitbox_state: &HitboxState,
    _: &RemoveOnHitComponent,
) {
    if !hitbox_state.hit_entities.is_empty() {
        cmd.remove(*entity);
    }
}
