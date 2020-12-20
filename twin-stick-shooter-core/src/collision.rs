use cgmath::{vec2, MetricSpace};
use std::fmt::Debug;
use std::ops::Add;

use crate::Vec2;

#[derive(Clone, Debug)]
/// An axis-aligned bounding box.
pub struct Aabb {
    pub min: Vec2,
    pub max: Vec2,
}

impl Add<Vec2> for Aabb {
    type Output = Aabb;

    fn add(self, rhs: Vec2) -> Aabb {
        Aabb {
            min: self.min + rhs,
            max: self.max + rhs,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Shape {
    Circle(Circle),
}

impl Shape {
    pub fn local_bounding_rect(&self) -> Aabb {
        match self {
            Shape::Circle(circle) => circle.local_bounding_rect(),
        }
    }

    pub fn test(shape_a: &Shape, pos_a: Vec2, shape_b: &Shape, pos_b: Vec2) -> bool {
        match (shape_a, shape_b) {
            (Shape::Circle(circle_a), Shape::Circle(circle_b)) => {
                pos_a.distance(pos_b) <= (circle_a.radius + circle_b.radius)
            }
        }
    }
}

impl From<Circle> for Shape {
    fn from(circle: Circle) -> Self {
        Shape::Circle(circle)
    }
}

#[derive(Clone, Debug)]
pub struct Circle {
    pub radius: f32,
}

impl Circle {
    fn local_bounding_rect(&self) -> Aabb {
        Aabb {
            min: vec2(-self.radius, -self.radius),
            max: vec2(self.radius, self.radius),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CollisionMask(u32);

impl CollisionMask {
    pub const TARGET: CollisionMask = CollisionMask(0x00000001);

    pub fn overlaps(self, rhs: CollisionMask) -> bool {
        (self.0 & rhs.0) != 0
    }
}
