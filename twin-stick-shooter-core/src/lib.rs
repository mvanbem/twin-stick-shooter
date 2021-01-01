use cgmath::vec3;

pub mod bullet;
pub mod collision;
pub mod game;
pub mod health;
pub mod hitbox;
pub mod interpolate;
pub mod model;
pub mod physics;
pub mod player;
pub mod position;
pub mod resource;
pub mod test;
pub mod util;

pub type Pt2 = cgmath::Point2<f32>;
pub type Vec2 = cgmath::Vector2<f32>;
pub type Mat3 = cgmath::Matrix3<f32>;

pub fn translation(x: Vec2) -> Mat3 {
    Mat3::from_cols(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), x.extend(1.0))
}
