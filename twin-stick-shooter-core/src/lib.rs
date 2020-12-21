use cgmath::num_traits::zero;
use legion::{Resources, Schedule, World};
use rand::{Rng, SeedableRng};
use rand_distr::Distribution;
use rand_pcg::Pcg32;

pub mod collision;
pub mod component;
pub mod resource;
pub mod system;
pub mod util;

use collision::{Circle, CollisionMask};
use component::{
    ForceAccumulator, Health, Hurtbox, InterpolatedPosition, Player, PlayerPlan, Position,
    PrevPosition, ReflectWithin, Velocity,
};
use resource::{Input, Subframe, Time};
use system::{
    collide_system, damage_system, interpolate_system, lifespan_system, physics_system,
    player_act_system, player_plan_system, reflect_within_system, remove_on_hit_system,
};
use util::Timer;

use crate::component::{HurtboxState, Inventory, Mass};

pub type Vec2 = cgmath::Vector2<f32>;

struct UnitDisc;

impl Distribution<Vec2> for UnitDisc {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec2 {
        rand_distr::UnitDisc.sample(rng).into()
    }
}

pub struct Game {
    world: World,
    step_resources: Resources,
    step_schedule: Schedule,
    interpolate_resources: Resources,
    interpolate_schedule: Schedule,
}

impl Game {
    pub fn new() -> Game {
        let mut seed = <Pcg32 as SeedableRng>::Seed::default();
        getrandom::getrandom(&mut seed[..]).unwrap_or_else(|_| {
            eprintln!("WARNING: getrandom() failed; proceeding with default random seed");
            ()
        });
        let mut rng = Pcg32::from_seed(seed);

        let mut world = World::default();

        // Create some targets.
        let mut targets = vec![];
        for _ in 0..32 {
            let pos = UnitDisc.sample(&mut rng) * 400.0;
            targets.push((
                Position(pos),
                PrevPosition(pos),
                InterpolatedPosition(pos),
                Velocity(UnitDisc.sample(&mut rng) * 100.0),
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

        Game {
            world,
            step_resources: Resources::default(),
            step_schedule: Schedule::builder()
                .add_system(player_plan_system())
                .add_system(physics_system())
                .add_system(reflect_within_system())
                .add_system(player_act_system())
                .add_system(collide_system())
                .add_system(damage_system())
                .add_system(lifespan_system())
                .add_system(remove_on_hit_system())
                .build(),
            interpolate_resources: Resources::default(),
            interpolate_schedule: Schedule::builder().add_system(interpolate_system()).build(),
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn step(&mut self, elapsed_seconds: f32, input: Input) {
        self.step_resources.insert(Time { elapsed_seconds });
        self.step_resources.insert(input);
        self.step_schedule
            .execute(&mut self.world, &mut self.step_resources);
    }

    pub fn interpolate(&mut self, subframe: Subframe) {
        self.interpolate_resources.insert(subframe);
        self.interpolate_schedule
            .execute(&mut self.world, &mut self.interpolate_resources);
    }
}
