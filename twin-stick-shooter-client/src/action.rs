use cgmath::num_traits::zero;
use rand_distr::Distribution;
use twin_stick_shooter_core::collision::{Circle, CollisionMask};
use twin_stick_shooter_core::component::{
    ForceAccumulator, Health, Hurtbox, HurtboxState, InterpolatedPosition, Inventory, Mass, Player,
    PlayerPlan, Position, PrevPosition, ReflectWithin, Velocity,
};
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
            PrevPosition(pos),
            InterpolatedPosition(pos),
            Velocity(UnitDisc.sample(rng) * 100.0),
            ForceAccumulator::default(),
            Mass::new(100.0),
            Hurtbox {
                shape: Circle { radius: 20.0 }.into(),
                mask: CollisionMask::TARGET,
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
        PrevPosition(zero()),
        InterpolatedPosition(zero()),
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
