use cgmath::num_traits::zero;
use rand::Rng;
use rand_distr::Distribution;
use twin_stick_shooter_core::collision::{Circle, CollisionMask};
use twin_stick_shooter_core::component::{
    ForceAccumulator, Health, Hurtbox, HurtboxState, InterpolatedPosition, Inventory, Mass, Player,
    PlayerPlan, Position, PrevPosition, ReflectWithin, Velocity,
};
use twin_stick_shooter_core::util::Timer;
use twin_stick_shooter_core::{Game, Vec2};

use crate::gui::{InvokeItemResult, Menu};

struct UnitDisc;

impl Distribution<Vec2> for UnitDisc {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec2 {
        rand_distr::UnitDisc.sample(rng).into()
    }
}

#[derive(Debug)]
pub struct MainMenu;

impl Menu for MainMenu {
    fn title(&self) -> &str {
        "this is a title screen"
    }

    fn items(&self) -> &[&str] {
        &["Online Multiplayer", "Single Player", "Third Option"]
    }

    fn invoke_item(&mut self, _index: usize, game: &mut Game) -> InvokeItemResult {
        // TODO: don't just disregard the index

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

        InvokeItemResult::ReplaceMenu(None)
    }
}
