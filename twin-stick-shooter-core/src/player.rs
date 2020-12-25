use cgmath::num_traits::zero;
use cgmath::InnerSpace;
use legion::systems::CommandBuffer;

use crate::bullet::{LifespanComponent, RemoveOnHitComponent};
use crate::collision::Circle;
use crate::hitbox::{Hitbox, HitboxMask, HitboxState};
use crate::interpolate::Interpolate;
use crate::physics::{ForceAccumulator, Mass, Velocity};
use crate::position::Position;
use crate::resource::{Input, Time};
use crate::util::Timer;
use crate::Vec2;

#[derive(Clone, Debug)]
pub struct Player {
    pub shoot_cooldown: Timer,
    pub inventory: Inventory,
}

#[derive(Clone, Debug, Default)]
pub struct PlayerPlan {
    pub shoot: Option<Vec2>,
}

// TODO: `Inventory` is not a Legion component, so maybe it should go somewhere else?
#[derive(Clone, Debug)]
pub struct Inventory {}

#[legion::system(for_each)]
pub fn player_plan(
    &Velocity(vel): &Velocity,
    ForceAccumulator(force): &mut ForceAccumulator,
    mass: &Mass,
    player: &mut Player,
    plan: &mut PlayerPlan,
    #[resource] time: &Time,
    #[resource] input: &Input,
) {
    let deadzoned_move = {
        let r = input.move_.magnitude();
        if r > 1.0 {
            input.move_.normalize()
        } else if r > 0.5 {
            input.move_.normalize_to((r - 0.5) * 2.0)
        } else {
            zero()
        }
    };
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
        plan.shoot = Some(input.aim);
        player.shoot_cooldown.reset(0.1);
    } else {
        plan.shoot = None;
    }
}

#[legion::system(for_each)]
pub fn player_act(
    cmd: &mut CommandBuffer,
    &Position(pos): &Position,
    &Velocity(vel): &Velocity,
    plan: &mut PlayerPlan,
) {
    if let Some(dir) = plan.shoot {
        let bullet_pos = pos + dir.normalize_to(20.0);
        cmd.push((
            Position(bullet_pos),
            Interpolate {
                prev_pos: bullet_pos,
                interpolated_pos: bullet_pos,
            },
            Velocity(vel + dir.normalize_to(1000.0)),
            LifespanComponent(Timer::with_remaining(1.0)),
            Hitbox {
                shape: Circle { radius: 5.0 }.into(),
                dbvt_index: None,
                mask: HitboxMask::TARGET,
                damage: 1.0,
            },
            HitboxState::default(),
            RemoveOnHitComponent,
        ));
    }
}
