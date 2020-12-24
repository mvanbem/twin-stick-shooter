use legion::{Resources, Schedule, World};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;

pub mod collision;
pub mod component;
pub mod resource;
pub mod system;
pub mod util;

use resource::{Input, Subframe, Time};
use system::{
    collide_system, damage_system, interpolate_system, lifespan_system, physics_system,
    player_act_system, player_plan_system, reflect_within_system, remove_on_hit_system,
};

pub type Vec2 = cgmath::Vector2<f32>;

pub struct Game {
    rng: Pcg32,
    world: World,
    is_paused: bool,

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
        });
        let rng = Pcg32::from_seed(seed);

        let world = World::default();

        Game {
            rng,
            world,
            is_paused: false,

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

    pub fn rng_mut(&mut self) -> &mut impl Rng {
        &mut self.rng
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn rng_and_world_mut(&mut self) -> (&mut impl Rng, &mut World) {
        (&mut self.rng, &mut self.world)
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn set_is_paused(&mut self, is_paused: bool) {
        self.is_paused = is_paused;
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

impl Default for Game {
    fn default() -> Self {
        Game::new()
    }
}
