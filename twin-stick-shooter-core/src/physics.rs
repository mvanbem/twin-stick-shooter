use cgmath::num_traits::zero;

use crate::interpolate::Interpolate;
use crate::position::Position;
use crate::resource::Time;
use crate::Vec2;

#[derive(Clone, Debug)]
pub struct ForceAccumulator(pub Vec2);

impl Default for ForceAccumulator {
    fn default() -> Self {
        ForceAccumulator(zero())
    }
}

#[derive(Clone, Debug)]
pub struct Mass {
    mass: f32,
    inv_mass: f32,
}

impl Mass {
    pub fn new(mass: f32) -> Mass {
        Mass {
            mass,
            inv_mass: 1.0 / mass,
        }
    }

    pub fn mass(&self) -> f32 {
        self.mass
    }

    pub fn inv_mass(&self) -> f32 {
        self.inv_mass
    }
}

#[derive(Clone, Debug)]
pub struct Velocity(pub Vec2);

#[legion::system(for_each)]
pub fn physics(
    #[resource] time: &Time,
    mass: Option<&Mass>,
    Position(pos): &mut Position,
    interpolate: Option<&mut Interpolate>,
    Velocity(vel): &mut Velocity,
    force: Option<&mut ForceAccumulator>,
) {
    if let (Some(mass), Some(ForceAccumulator(force))) = (mass, force) {
        *vel += *force * mass.inv_mass() * time.elapsed_seconds;
        *force = zero();
    }
    if let Some(Interpolate { prev_pos, .. }) = interpolate {
        *prev_pos = *pos;
    }
    *pos += *vel * time.elapsed_seconds;
}
