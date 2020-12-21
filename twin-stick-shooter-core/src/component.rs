use cgmath::num_traits::zero;
use legion::Entity;

use crate::collision::{CollisionMask, Shape};
use crate::util::Timer;
use crate::Vec2;

mod mass;
mod player;

pub use mass::Mass;
pub use player::{Inventory, Player, PlayerPlan};

#[derive(Clone, Debug)]
pub struct ForceAccumulator(pub Vec2);

impl Default for ForceAccumulator {
    fn default() -> Self {
        ForceAccumulator(zero())
    }
}

#[derive(Clone, Debug)]
pub struct Health(pub f32);

/// A collider that deals damage.
#[derive(Clone, Debug)]
pub struct Hitbox {
    pub shape: Shape,
    pub mask: CollisionMask,
    pub damage: f32,
}

#[derive(Clone, Debug, Default)]
pub struct HitboxState {
    pub hit_entities: Vec<Entity>,
}

/// A collider that receives damage.
#[derive(Clone, Debug)]
pub struct Hurtbox {
    pub shape: Shape,
    pub mask: CollisionMask,
}

#[derive(Clone, Debug, Default)]
pub struct HurtboxState {
    pub hit_by_entities: Vec<Entity>,
}

#[derive(Clone, Debug)]
pub struct Lifespan(pub Timer);

#[derive(Clone, Debug)]
pub struct Position(pub Vec2);

#[derive(Clone, Debug)]
pub struct PrevPosition(pub Vec2);

#[derive(Clone, Debug)]
pub struct InterpolatedPosition(pub Vec2);

#[derive(Clone, Debug)]
pub struct ReflectWithin(pub f32);

#[derive(Clone, Debug)]
pub struct RemoveOnHit;

#[derive(Clone, Debug)]
pub struct Velocity(pub Vec2);
