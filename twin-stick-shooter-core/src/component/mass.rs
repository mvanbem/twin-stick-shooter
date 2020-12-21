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
