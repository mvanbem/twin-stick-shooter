use cgmath::vec2;
use legion::{Resources, Schedule, World};
use twin_stick_shooter_core::collision::{Circle, CollisionMask, Shape};
use twin_stick_shooter_core::component::{Hitbox, HitboxState, Hurtbox, HurtboxState, Position};
use twin_stick_shooter_core::system::collide_system;

#[test]
fn non_overlapping() {
    let shape_a = Shape::Circle(Circle { radius: 2.0 });
    let pos_a = vec2(-2.1, 0.0);
    let shape_b = Shape::Circle(Circle { radius: 3.0 });
    let pos_b = vec2(3.1, 0.0);
    assert_eq!(Shape::test(&shape_a, pos_a, &shape_b, pos_b), false);
}

#[test]
fn overlapping() {
    let shape_a = Shape::Circle(Circle { radius: 2.0 });
    let pos_a = vec2(-1.9, 0.0);
    let shape_b = Shape::Circle(Circle { radius: 3.0 });
    let pos_b = vec2(2.9, 0.0);
    assert_eq!(Shape::test(&shape_a, pos_a, &shape_b, pos_b), true);
}

#[test]
fn system() {
    let mut world = World::default();
    let hitbox_circle = world.push((
        Position(vec2(-2.0, 0.0)),
        Hitbox {
            shape: Shape::Circle(Circle { radius: 2.0 }),
            mask: CollisionMask::TARGET,
            damage: 1.0,
        },
        HitboxState::default(),
    ));
    let hurtbox_circle_should_hit = world.push((
        Position(vec2(2.9, 0.0)),
        Hurtbox {
            shape: Shape::Circle(Circle { radius: 3.0 }),
            mask: CollisionMask::TARGET,
        },
        HurtboxState::default(),
    ));
    let hurtbox_circle_should_miss = world.push((
        Position(vec2(3.1, 0.0)),
        Hurtbox {
            shape: Shape::Circle(Circle { radius: 3.0 }),
            mask: CollisionMask::TARGET,
        },
        HurtboxState::default(),
    ));
    let mut resources = Resources::default();
    let mut schedule = Schedule::builder().add_system(collide_system()).build();
    schedule.execute(&mut world, &mut resources);

    assert_eq!(
        world
            .entry(hitbox_circle)
            .unwrap()
            .get_component::<HitboxState>()
            .unwrap()
            .hit_entities,
        &[hurtbox_circle_should_hit]
    );
    assert_eq!(
        world
            .entry(hurtbox_circle_should_hit)
            .unwrap()
            .get_component::<HurtboxState>()
            .unwrap()
            .hit_by_entities,
        &[hitbox_circle],
    );
    assert_eq!(
        world
            .entry(hurtbox_circle_should_miss)
            .unwrap()
            .get_component::<HurtboxState>()
            .unwrap()
            .hit_by_entities,
        &[],
    );
}
