use cgmath::num_traits::{clamp, one};
use cgmath::InnerSpace;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::{Entity, EntityStore, IntoQuery};

use crate::bullet::{LifespanComponent, RemoveOnHitComponent};
use crate::collision::Circle;
use crate::hitbox::{HitboxComponent, HitboxEffect, HitboxMask, HurtboxComponent};
use crate::interpolate::InterpolateComponent;
use crate::model::ModelComponent;
use crate::physics::{ForceComponent, MassComponent, VelocityComponent};
use crate::position::PositionComponent;
use crate::resource::{GuiOverride, GuiOverrideQueue, Input, Time};
use crate::util::{map_magnitude, Timer};
use crate::Vec2;

#[derive(Clone, Debug)]
pub struct PlayerComponent {
    // Attributes.
    pub shoot_cooldown: Timer,
    pub inventory: Inventory,
    pub docked_to: Option<Entity>,

    // Intra-frame state.
    pub shoot: Option<Vec2>,
}

#[derive(Clone, Debug)]
pub struct Inventory {}

#[legion::system]
#[read_component(PositionComponent)]
#[read_component(VelocityComponent)]
#[read_component(MassComponent)]
#[write_component(ForceComponent)]
#[write_component(PlayerComponent)]
pub fn player_plan(world: &mut SubWorld, #[resource] time: &Time, #[resource] input: &Input) {
    let mut player_query = <(
        &VelocityComponent,
        &mut ForceComponent,
        &MassComponent,
        &mut PlayerComponent,
    )>::query();
    let (mut player_world, pos_world) = world.split_for_query(&player_query);

    for (
        entity,
        (&VelocityComponent(vel), &mut ForceComponent(ref mut force), ref mass, ref mut player),
    ) in player_query
        .iter_chunks_mut(&mut player_world)
        .flat_map(|chunk| chunk.into_iter_entities())
    {
        if let Some(station) = player.docked_to {
            let &PositionComponent(pos) = pos_world
                .entry_ref(entity)
                .unwrap()
                .into_component()
                .unwrap();
            let &PositionComponent(station_pos) = pos_world
                .entry_ref(station)
                .unwrap()
                .into_component()
                .unwrap();

            *force += 1e4 * (station_pos - pos);

            player.shoot = None;
        } else {
            let deadzoned_move = map_magnitude(input.move_, |r| clamp((r - 0.5) * 2.0, 0.0, 1.0));
            let goal_vel = 250.0 * deadzoned_move;
            let goal_force = (goal_vel - vel) * mass.mass() / time.elapsed_seconds;
            *force += {
                let r = goal_force.magnitude();
                const MAX_FORCE: f32 = 1.0e6;
                if r < MAX_FORCE {
                    goal_force
                } else {
                    goal_force.normalize_to(MAX_FORCE)
                }
            };

            if player.shoot_cooldown.step_and_is_elapsed(time) && input.fire {
                // TODO: shoot only when the player's shoot input is active
                player.shoot = Some(input.aim);
                player.shoot_cooldown.reset(0.1);
            } else {
                player.shoot = None;
            }
        }
    }
}

#[legion::system(for_each)]
pub fn player_act(
    cmd: &mut CommandBuffer,
    &PositionComponent(pos): &PositionComponent,
    &VelocityComponent(vel): &VelocityComponent,
    player: &mut PlayerComponent,
) {
    if let Some(dir) = player.shoot {
        let bullet_pos = pos + dir.normalize_to(20.0);
        cmd.push((
            PositionComponent(bullet_pos),
            InterpolateComponent {
                prev_pos: bullet_pos,
                interpolated_pos: bullet_pos,
            },
            VelocityComponent(vel + dir.normalize_to(1000.0)),
            LifespanComponent(Timer::with_remaining(1.0)),
            HitboxComponent {
                shape: Circle { radius: 5.0 }.into(),
                dbvt_index: None,
                mask: HitboxMask::TARGET,
                effect: HitboxEffect::Damage(1.0),
                hit_entities: vec![],
            },
            RemoveOnHitComponent,
            ModelComponent {
                name: "shots/lemon".to_string(),
                transform: one(),
            },
        ));
    }
}

#[legion::system(for_each)]
#[read_component(HitboxComponent)]
pub fn player_react(
    world: &SubWorld,
    hurtbox: &HurtboxComponent,
    player: &mut PlayerComponent,
    #[resource] gui_override_queue: &GuiOverrideQueue,
) {
    for hitbox_entity in hurtbox.hit_by_entities.iter().copied() {
        let hitbox: &HitboxComponent = world
            .entry_ref(hitbox_entity)
            .unwrap()
            .into_component::<HitboxComponent>()
            .unwrap();
        if let HitboxEffect::StationDock = hitbox.effect {
            player.docked_to = Some(hitbox_entity);
            gui_override_queue.push_back(GuiOverride::StationDocked);
        }
    }
}
