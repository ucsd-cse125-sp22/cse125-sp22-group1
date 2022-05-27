use std::time::Instant;

use crate::physics::{physics_changes::PhysicsChangeType, stats_changes::StatsChange};
use chariot_core::{player::choices::Stat, questions::AudienceAction};

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

        // ??? probably need to be special-case handled
        AudienceAction::ShoppingCart => todo!(),
        AudienceAction::SpeedBalanceBoost => todo!(),
        AudienceAction::Backwards => todo!(),

        AudienceAction::SwapFirstAndLast => None,
        AudienceAction::ShufflePlayerPositions => None,
        AudienceAction::ResetLapCounter => None,

        _ => None,
    }
}

pub fn get_stats_change_from_audience_action(
    audience_action: &AudienceAction,
    expiration_time: Instant,
) -> Option<StatsChange> {
    let stat_and_multiplier: Option<(Stat, f64)> = match audience_action {
        AudienceAction::DoubleMaxSpeed => Some((Stat::MaxCarSpeed, 2.0)),
        AudienceAction::SuperAccelerator => Some((Stat::CarAccelerator, 3.0)),
        AudienceAction::SuperSpin => Some((Stat::CarSpin, 5.0)),
        AudienceAction::MoonGravity => Some((Stat::GravityCoefficient, 0.25)),
        AudienceAction::IceRink => Some((Stat::RollingResistanceCoefficient, 0.0)),
        AudienceAction::ExplosivePlayerCollisions => Some((Stat::PlayerBounciness, 3.0)),
        AudienceAction::SuperBouncyObjects => Some((Stat::TerrainBounciness, 3.0)),

        _ => None,
    };

    if let Some((stat, multiplier)) = stat_and_multiplier {
        Some(StatsChange {
            stat,
            multiplier,
            expiration_time,
        })
    } else {
        None
    }
}
