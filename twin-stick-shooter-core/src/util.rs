use cgmath::{BaseFloat, InnerSpace, VectorSpace};

mod timer;

pub use timer::Timer;

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
