use cgmath::num_traits::zero;
use cgmath::{vec2, BaseFloat, InnerSpace, VectorSpace};
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
    ForceAccumulator, Health, Hurtbox, Player, PlayerPlan, Position, ReflectWithin, Velocity,
};
use resource::{Input, Time};
use system::{
    collide_system, damage_system, lifespan_system, physics_system, player_act_system,
    player_plan_system, reflect_within_system, remove_on_hit_system,
};
use util::Timer;

use crate::component::{HurtboxState, Mass};

pub type Vec2 = cgmath::Vector2<f32>;

struct UnitDisc;

impl Distribution<Vec2> for UnitDisc {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec2 {
        rand_distr::UnitDisc.sample(rng).into()
    }
}

pub struct Game {
    world: World,
    resources: Resources,
    step_schedule: Schedule,
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
            targets.push((
                Position(UnitDisc.sample(&mut rng) * 400.0),
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
            Velocity(zero()),
            ForceAccumulator::default(),
            Mass::new(100.0),
            Player {
                shoot_cooldown: Timer::elapsed(),
            },
            PlayerPlan::default(),
        ));

        let mut resources = Resources::default();
        resources.insert(Time {
            elapsed_seconds: 1.0 / 60.0,
        });

        let step_schedule = Schedule::builder()
            .add_system(player_plan_system())
            .add_system(physics_system())
            .add_system(reflect_within_system())
            .add_system(player_act_system())
            .add_system(collide_system())
            .add_system(damage_system())
            .add_system(lifespan_system())
            .add_system(remove_on_hit_system())
            .build();

        Game {
            world,
            resources,
            step_schedule,
        }
    }

    pub fn world(&mut self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn step(&mut self, elapsed_seconds: f32, input: Input) {
        self.resources.insert(Time { elapsed_seconds });
        self.resources.insert(input);
        self.step_schedule
            .execute(&mut self.world, &mut self.resources);
    }
}

fn seek<V: InnerSpace>(x: V, goal: V, max_abs_dx: <V as VectorSpace>::Scalar) -> V
where
    <V as VectorSpace>::Scalar: BaseFloat,
{
    let dx = goal - x;
    let abs_dx = dx.magnitude();
    if abs_dx <= max_abs_dx {
        x
    } else {
        x + dx.normalize_to(max_abs_dx)
    }
}
