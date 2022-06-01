use std::time::Instant;

use crate::physics::{
    physics_changes::PhysicsChangeType, player_entity::PlayerEntity, stats_changes::StatsChange,
};
use chariot_core::{
    player::{choices::Stat, lap_info::LapInformation},
    questions::AudienceAction,
};
use glam::DVec3;

pub fn get_physics_change_from_audience_action(
    audience_action: &AudienceAction,
) -> Option<PhysicsChangeType> {
    match audience_action {
        AudienceAction::Null => None,
        // controls-only audience actions
        AudienceAction::NoLeft => Some(PhysicsChangeType::NoTurningLeft),
        AudienceAction::NoRight => Some(PhysicsChangeType::NoTurningRight),
        AudienceAction::InvertControls => Some(PhysicsChangeType::InvertedControls),
        AudienceAction::AutoAccelerate => Some(PhysicsChangeType::AutoAccelerate),
        AudienceAction::TurnOnlyWhenNotMoving => Some(PhysicsChangeType::TurnOnlyWhenNotMoving),

        // physics-affecting (not just controls)
        AudienceAction::ShoppingCart => Some(PhysicsChangeType::ShoppingCart),
        AudienceAction::SpeedBalanceBoost => Some(PhysicsChangeType::SpeedBalanceBoost),

        _ => None,
    }
}

pub fn get_stats_changes_from_audience_action(
    audience_action: &AudienceAction,
    expiration_time: Instant,
) -> Vec<StatsChange> {
    let stats_and_multipliers: Vec<(Stat, f64)> = match audience_action {
        AudienceAction::DoubleMaxSpeed => vec![(Stat::MaxCarSpeed, 2.0)],
        AudienceAction::SuperAccelerator => vec![(Stat::CarAccelerator, 3.0)],
        AudienceAction::SuperSpin => vec![
            (Stat::CarSpin, 2.5),
            (Stat::MaxCarSpin, 7.5),
            (Stat::RotationReductionCoefficient, 2.0),
        ],
        AudienceAction::MoonGravity => vec![(Stat::GravityCoefficient, 0.25)],
        AudienceAction::IceRink => vec![(Stat::RollingResistanceCoefficient, 0.0)],
        AudienceAction::ExplosivePlayerCollisions => vec![(Stat::PlayerBounciness, 3.0)],
        AudienceAction::SuperBouncyObjects => vec![(Stat::TerrainBounciness, 3.0)],

        _ => vec![],
    };

    stats_and_multipliers
        .iter()
        .map(|(stat, multiplier)| StatsChange {
            stat: *stat,
            multiplier: *multiplier,
            expiration_time,
        })
        .collect()
}

pub fn handle_one_time_audience_action(
    audience_action: &AudienceAction,
    players: &mut [PlayerEntity; 4],
) {
    match audience_action {
        AudienceAction::Backwards => {
            let backwards_rotation = glam::DQuat::from_axis_angle(DVec3::Y, std::f64::consts::PI);
            for player in players {
                player.entity_location.unit_steer_direction =
                    backwards_rotation * player.entity_location.unit_steer_direction;
                player.velocity = backwards_rotation * player.velocity;
            }
        }

        AudienceAction::SwapFirstAndLast => {
            let first_idx = players
                .iter()
                .position(|player| player.lap_info.placement == 1);
            let last_idx = players
                .iter()
                .position(|player| player.lap_info.placement == 4);

            // just do nothing if there's a tie or whatever, that's better than
            // crashing on an unwrap
            if first_idx.is_some() && last_idx.is_some() {
                let first_idx = first_idx.unwrap();
                let last_idx = last_idx.unwrap();

                let new_last_position = players[first_idx].entity_location.position;
                let new_last_lap_info = players[first_idx].lap_info;

                players[first_idx].entity_location.position =
                    players[last_idx].entity_location.position;
                players[first_idx].lap_info = players[last_idx].lap_info;

                players[last_idx].entity_location.position = new_last_position;
                players[last_idx].lap_info = new_last_lap_info;
            }
        }

        AudienceAction::ShufflePlayerPositions => {
            // we want everyone to go to a different placement, so we'll just
            // manually hardcode the allowable shuffle orders
            // lmao
            let shuffle_order: [usize; 4] = [
                [1, 0, 3, 2],
                [1, 2, 3, 0],
                [1, 3, 0, 2],
                [2, 0, 3, 1],
                [2, 3, 0, 1],
                [2, 3, 1, 0],
                [3, 0, 1, 2],
                [3, 2, 1, 0],
                [3, 2, 0, 1],
            ][(9.0 * rand::random::<f64>()).floor() as usize];

            let positions: Vec<DVec3> = shuffle_order
                .iter()
                .map(|&i| players[i].entity_location.position)
                .collect();
            let lap_infos: Vec<LapInformation> =
                shuffle_order.iter().map(|&i| players[i].lap_info).collect();

            for (i, player) in players.iter_mut().enumerate() {
                player.entity_location.position = positions.get(i).unwrap().to_owned();
                player.lap_info = lap_infos.get(i).unwrap().to_owned();
            }
        }

        AudienceAction::ResetLapCounter => {
            for player in players.iter_mut() {
                player.lap_info.lap = 1;
            }
        }

        _ => {}
    }
}
