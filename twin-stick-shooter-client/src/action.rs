use cgmath::num_traits::zero;
use rand_distr::Distribution;
use twin_stick_shooter_core::collision::Circle;
use twin_stick_shooter_core::health::Health;
use twin_stick_shooter_core::hitbox::{HitboxMask, Hurtbox, HurtboxState};
use twin_stick_shooter_core::interpolate::Interpolate;
use twin_stick_shooter_core::physics::{ForceAccumulator, Mass, Velocity};
use twin_stick_shooter_core::player::{Inventory, Player, PlayerPlan};
use twin_stick_shooter_core::position::Position;
use twin_stick_shooter_core::test::ReflectWithin;
use twin_stick_shooter_core::util::{Timer, UnitDisc};
use twin_stick_shooter_core::Game;

pub fn create_game(game: &mut Game) {
    let (rng, world) = game.rng_and_world_mut();
    world.clear();

    // Create some targets.
    let mut targets = vec![];
    for _ in 0..32 {
        let pos = UnitDisc.sample(rng) * 400.0;
        targets.push((
            Position(pos),
            Interpolate {
                prev_pos: pos,
                interpolated_pos: pos,
            },
            Velocity(UnitDisc.sample(rng) * 100.0),
            ForceAccumulator::default(),
            Mass::new(100.0),
            Hurtbox {
                shape: Circle { radius: 20.0 }.into(),
                dbvt_index: None,
                mask: HitboxMask::TARGET,
            },
            HurtboxState::default(),
            Health(3.0),
            ReflectWithin(400.0),
        ));
    }
    world.extend(targets);

    // Create a player entity.
    world.push((
        Position(zero()),
        Interpolate {
            prev_pos: zero(),
            interpolated_pos: zero(),
        },
        Velocity(zero()),
        ForceAccumulator::default(),
        Mass::new(100.0),
        Player {
            shoot_cooldown: Timer::elapsed(),
            inventory: Inventory {},
        },
        PlayerPlan::default(),
    ));
}
