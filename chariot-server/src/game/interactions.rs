use crate::physics::physics_changes::PhysicsChangeType;
use chariot_core::questions::AudienceAction;

pub fn get_physics_change_from_audience_action(
    audience_action: &AudienceAction,
) -> Option<PhysicsChangeType> {
    match audience_action {
        AudienceAction::Null => None,
        AudienceAction::NoLeft => Some(PhysicsChangeType::NoTurningLeft),
        AudienceAction::NoRight => Some(PhysicsChangeType::NoTurningRight),
        AudienceAction::InvertControls => Some(PhysicsChangeType::InvertedControls),
        AudienceAction::AutoAccelerate => Some(PhysicsChangeType::AutoAccelerate),
        AudienceAction::TurnOnlyWhenNotMoving => Some(PhysicsChangeType::TurnOnlyWhenNotMoving),
        AudienceAction::SwapFirstAndLast => todo!(),
        AudienceAction::ShufflePlayerPositions => todo!(),
        AudienceAction::DoubleMaxSpeed => todo!(),
        AudienceAction::SuperAccelerator => todo!(),
        AudienceAction::SuperSpin => todo!(),
        AudienceAction::ShoppingCart => todo!(),
        AudienceAction::MoonGravity => todo!(),
        AudienceAction::IceRink => todo!(),
        AudienceAction::ExplosivePlayerCollisions => todo!(),
        AudienceAction::SuperBouncyObjects => todo!(),
        AudienceAction::SpeedBalanceBoost => todo!(),
        AudienceAction::ResetLapCounter => todo!(),
        AudienceAction::Backwards => todo!(),
        _ => None,
    }
}
