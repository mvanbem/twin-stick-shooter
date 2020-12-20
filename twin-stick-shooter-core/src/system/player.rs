use cgmath::num_traits::zero;
use cgmath::InnerSpace;
use legion::systems::CommandBuffer;

use crate::collision::{Circle, CollisionMask};
use crate::component::{
    ForceAccumulator, Hitbox, HitboxState, Lifespan, Mass, Player, PlayerPlan, Position,
    RemoveOnHit, Velocity,
};
use crate::resource::{Input, Time};
use crate::util::Timer;

#[legion::system(for_each)]
pub fn player_plan(
    vel: &Velocity,
    force: &mut ForceAccumulator,
    mass: &Mass,
    player: &mut Player,
    plan: &mut PlayerPlan,
    #[resource] time: &Time,
    #[resource] input: &Input,
) {
    let deadzoned_move = {
        let r = input.move_.magnitude();
        if r > 0.5 {
            input.move_.normalize_to((r - 0.5) * 2.0)
        } else {
            zero()
        }
    };
    let goal_vel = 250.0 * deadzoned_move;
    let goal_force = (goal_vel - vel.0) * mass.mass() / time.elapsed_seconds;
    force.0 += {
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
pub fn player_act(cmd: &mut CommandBuffer, pos: &Position, vel: &Velocity, plan: &mut PlayerPlan) {
    if let Some(dir) = plan.shoot {
        cmd.push((
            Position(pos.0 + dir.normalize_to(20.0)),
            Velocity(vel.0 + dir.normalize_to(1000.0)),
            Lifespan(Timer::with_remaining(2.0)),
            Hitbox {
                shape: Circle { radius: 5.0 }.into(),
                mask: CollisionMask::TARGET,
                damage: 1.0,
            },
            HitboxState::default(),
            RemoveOnHit,
        ));
    }
}