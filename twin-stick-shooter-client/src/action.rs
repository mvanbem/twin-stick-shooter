use cgmath::num_traits::{one, zero};
use cgmath::{vec2, EuclideanSpace};
use legion::{EntityStore, IntoQuery};
use rand_distr::Distribution;
use twin_stick_shooter_core::collision::Circle;
use twin_stick_shooter_core::game::Game;
use twin_stick_shooter_core::health::HealthComponent;
use twin_stick_shooter_core::hitbox::{
    HitboxComponent, HitboxEffect, HitboxMask, HurtboxComponent,
};
use twin_stick_shooter_core::interpolate::InterpolateComponent;
use twin_stick_shooter_core::model::ModelComponent;
use twin_stick_shooter_core::physics::{ForceComponent, MassComponent, VelocityComponent};
use twin_stick_shooter_core::player::{Inventory, PlayerComponent};
use twin_stick_shooter_core::position::PositionComponent;
use twin_stick_shooter_core::test::ReflectWithin;
use twin_stick_shooter_core::util::{Timer, UnitDisc};
use twin_stick_shooter_core::Pt2;

pub fn create_game(game: &mut Game) {
    let (rng, world) = game.rng_and_world_mut();
    world.clear();

    // Create some targets.
    let mut targets = vec![];
    for _ in 0..32 {
        let pos = Pt2::from_vec(UnitDisc.sample(rng) * 400.0);
        targets.push((
            PositionComponent(pos),
            InterpolateComponent {
                prev_pos: pos,
                interpolated_pos: pos,
            },
            VelocityComponent(UnitDisc.sample(rng) * 100.0),
            ForceComponent::default(),
            MassComponent::new(100.0),
            HurtboxComponent {
                shape: Circle { radius: 20.0 }.into(),
                dbvt_index: None,
                mask: HitboxMask::TARGET,
                hit_by_entities: vec![],
            },
            HealthComponent::new(3.0),
            ReflectWithin(400.0),
            ModelComponent {
                name: "test/target".to_string(),
                transform: one(),
            },
        ));
    }
    world.extend(targets);

    // Create a player entity.
    let pos = Pt2::from_vec(zero());
    world.push((
        PositionComponent(pos),
        InterpolateComponent {
            prev_pos: pos,
            interpolated_pos: pos,
        },
        VelocityComponent(zero()),
        ForceComponent::default(),
        MassComponent::new(100.0),
        HurtboxComponent {
            shape: Circle { radius: 20.0 }.into(),
            dbvt_index: None,
            mask: HitboxMask::PLAYER,
            hit_by_entities: vec![],
        },
        PlayerComponent {
            shoot_cooldown: Timer::elapsed(),
            inventory: Inventory {},
            docked_to: None,
            shoot: None,
        },
        ModelComponent {
            name: "ships/player".to_string(),
            transform: one(),
        },
    ));

    // Create a station entity.
    let pos = Pt2::from_vec(vec2(-400.0, 0.0));
    world.push((
        PositionComponent(pos),
        InterpolateComponent {
            prev_pos: pos,
            interpolated_pos: pos,
        },
        VelocityComponent(zero()),
        ForceComponent::default(),
        MassComponent::new(1e5),
        HitboxComponent {
            shape: Circle { radius: 50.0 }.into(),
            dbvt_index: None,
            mask: HitboxMask::PLAYER,
            effect: HitboxEffect::StationDock,
            hit_entities: vec![],
        },
        ModelComponent {
            name: "ships/station".to_string(),
            transform: one(),
        },
    ));
}

pub fn launch_from_station(game: &mut Game) {
    let mut player_query = <&mut PlayerComponent>::query();
    let (mut player_world, mut misc_world) = game.world_mut().split_for_query(&player_query);
    for (player_entity, player) in player_query
        .iter_chunks_mut(&mut player_world)
        .flat_map(|chunk| chunk.into_iter_entities())
    {
        if let Some(station_entity) = player.docked_to {
            // Snap the player just beyond the station.
            let &PositionComponent(station_pos) = misc_world
                .entry_ref(station_entity)
                .unwrap()
                .into_component()
                .unwrap();
            *misc_world
                .entry_mut(player_entity)
                .unwrap()
                .into_component_mut()
                .unwrap() = PositionComponent(station_pos + vec2(100.0, 0.0));

            // This player is no longer docked.
            player.docked_to = None;
        }
    }
}
