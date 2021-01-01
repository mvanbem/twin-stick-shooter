use cgmath::num_traits::zero;

use crate::interpolate::InterpolateComponent;
use crate::position::PositionComponent;
use crate::resource::Time;
use crate::Vec2;

#[derive(Clone, Debug)]
pub struct ForceComponent(pub Vec2);

impl Default for ForceComponent {
    fn default() -> Self {
        ForceComponent(zero())
    }
}

#[derive(Clone, Debug)]
pub struct MassComponent {
    mass: f32,
    inv_mass: f32,
}

impl MassComponent {
    pub fn new(mass: f32) -> MassComponent {
        MassComponent {
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
pub struct VelocityComponent(pub Vec2);

#[legion::system(for_each)]
pub fn physics(
    #[resource] time: &Time,
    mass: Option<&MassComponent>,
    PositionComponent(pos): &mut PositionComponent,
    interpolate: Option<&mut InterpolateComponent>,
    VelocityComponent(vel): &mut VelocityComponent,
    force: Option<&mut ForceComponent>,
) {
    if let (Some(mass), Some(ForceComponent(force))) = (mass, force) {
        *vel += *force * mass.inv_mass() * time.elapsed_seconds;
        *force = zero();
    }
    if let Some(InterpolateComponent { prev_pos, .. }) = interpolate {
        *prev_pos = *pos;
    }
    *pos += *vel * time.elapsed_seconds;
}
