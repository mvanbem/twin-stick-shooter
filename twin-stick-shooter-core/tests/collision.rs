use cgmath::{vec2, EuclideanSpace};
use collision::dbvt::DynamicBoundingVolumeTree;
use legion::{Resources, Schedule, World};
use rand::SeedableRng;
use rand_pcg::Pcg32;
use twin_stick_shooter_core::collision::{Circle, Shape};
use twin_stick_shooter_core::hitbox::{
    hitbox_system, HitboxComponent, HitboxEffect, HitboxMask, HurtboxComponent,
};
use twin_stick_shooter_core::position::PositionComponent;
use twin_stick_shooter_core::resource::CollideCounters;
use twin_stick_shooter_core::{translation, Pt2};

#[test]
fn non_overlapping() {
    let shape_a = Shape::Circle(Circle { radius: 2.0 });
    let xform_a = translation(vec2(-2.1, 0.0));
    let shape_b = Shape::Circle(Circle { radius: 3.0 });
    let xform_b = translation(vec2(3.1, 0.0));
    assert_eq!(
        twin_stick_shooter_core::collision::test(&shape_a, &xform_a, &shape_b, &xform_b),
        false
    );
}

#[test]
fn overlapping() {
    let shape_a = Shape::Circle(Circle { radius: 2.0 });
    let xform_a = translation(vec2(-1.9, 0.0));
    let shape_b = Shape::Circle(Circle { radius: 3.0 });
    let xform_b = translation(vec2(2.9, 0.0));
    assert_eq!(
        twin_stick_shooter_core::collision::test(&shape_a, &xform_a, &shape_b, &xform_b),
        true
    );
}

#[test]
fn system() {
    let mut world = World::default();
    let hitbox_circle = world.push((
        PositionComponent(Pt2::from_vec(vec2(-2.0, 0.0))),
        HitboxComponent {
            shape: Shape::Circle(Circle { radius: 2.0 }),
            dbvt_index: None,
            mask: HitboxMask::TARGET,
            effect: HitboxEffect::None,
            hit_entities: vec![],
        },
    ));
    let hurtbox_circle_should_hit = world.push((
        PositionComponent(Pt2::from_vec(vec2(2.9, 0.0))),
        HurtboxComponent {
            shape: Shape::Circle(Circle { radius: 3.0 }),
            dbvt_index: None,
            mask: HitboxMask::TARGET,
            hit_by_entities: vec![],
        },
    ));
    let hurtbox_circle_should_miss = world.push((
        PositionComponent(Pt2::from_vec(vec2(3.1, 0.0))),
        HurtboxComponent {
            shape: Shape::Circle(Circle { radius: 3.0 }),
            dbvt_index: None,
            mask: HitboxMask::TARGET,
            hit_by_entities: vec![],
        },
    ));
    let mut resources = Resources::default();
    resources.insert(Pcg32::from_rng(rand::thread_rng()).unwrap());
    resources.insert(CollideCounters::default());
    let mut schedule = Schedule::builder()
        .add_system(hitbox_system(DynamicBoundingVolumeTree::new()))
        .build();
    schedule.execute(&mut world, &mut resources);

    assert_eq!(
        world
            .entry(hitbox_circle)
            .unwrap()
            .get_component::<HitboxComponent>()
            .unwrap()
            .hit_entities,
        &[hurtbox_circle_should_hit]
    );
    assert_eq!(
        world
            .entry(hurtbox_circle_should_hit)
            .unwrap()
            .get_component::<HurtboxComponent>()
            .unwrap()
            .hit_by_entities,
        &[hitbox_circle],
    );
    assert_eq!(
        world
            .entry(hurtbox_circle_should_miss)
            .unwrap()
            .get_component::<HurtboxComponent>()
            .unwrap()
            .hit_by_entities,
        &[],
    );
}
