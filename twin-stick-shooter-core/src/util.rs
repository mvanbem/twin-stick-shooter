use cgmath::{BaseFloat, InnerSpace, VectorSpace};

mod timer;

use rand::Rng;
use rand_distr::Distribution;
pub use timer::Timer;

use crate::Vec2;

pub fn clamp_magnitude<V: InnerSpace>(
    v: V,
    a: <V as VectorSpace>::Scalar,
    b: <V as VectorSpace>::Scalar,
) -> V
where
    <V as VectorSpace>::Scalar: BaseFloat,
{
    let r = v.magnitude();
    if r < a {
        v.normalize_to(a)
    } else if r <= b {
        v
    } else {
        v.normalize_to(b)
    }
}

pub struct UnitDisc;

impl Distribution<Vec2> for UnitDisc {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec2 {
        rand_distr::UnitDisc.sample(rng).into()
    }
}
