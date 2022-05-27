use chariot_core::player::{
    choices::Stat,
    player_inputs::{EngineStatus, RotationStatus},
};
use glam::DVec3;

use super::player_entity::PlayerEntity;

use std::time::Instant;

#[derive(Clone)]
pub enum PhysicsChangeType {
    NoTurningRight,
    NoTurningLeft,
    InvertedControls,
    AutoAccelerate,
    TurnOnlyWhenNotMoving,
    IAmSpeed,
    ShoppingCart,
    InSpainButTheAIsSilent,
}

#[derive(Clone)]
pub struct PhysicsChange {
    pub change_type: PhysicsChangeType,
    pub expiration_time: Instant,
}

impl PlayerEntity {
    pub fn change_inputs_per_physics_changes(&mut self) {
        for change in &self.physics_changes {
            match change.change_type {
                PhysicsChangeType::NoTurningRight => {
                    if matches!(
                        self.player_inputs.rotation_status,
                        RotationStatus::InSpinClockwise { .. }
                    ) {
                        self.player_inputs.rotation_status = RotationStatus::NotInSpin;
                        self.angular_velocity -= self.stat(Stat::CarSpin);
                    }
                }

                PhysicsChangeType::NoTurningLeft => {
                    if matches!(
                        self.player_inputs.rotation_status,
                        RotationStatus::InSpinCounterclockwise { .. }
                    ) {
                        self.player_inputs.rotation_status = RotationStatus::NotInSpin;
                        self.angular_velocity -= self.stat(Stat::CarSpin);
                    }
                }

                PhysicsChangeType::InvertedControls => {
                    match self.player_inputs.engine_status {
                        EngineStatus::Accelerating(_) => {
                            self.player_inputs.engine_status = EngineStatus::Braking;
                        }
                        EngineStatus::Neutral => {}
                        EngineStatus::Braking => {
                            self.player_inputs.engine_status = EngineStatus::Accelerating(1.0);
                        }
                    }

                    match self.player_inputs.rotation_status {
                        RotationStatus::InSpinClockwise(x) => {
                            self.player_inputs.rotation_status =
                                RotationStatus::InSpinCounterclockwise(x)
                        }
                        RotationStatus::InSpinCounterclockwise(x) => {
                            self.player_inputs.rotation_status = RotationStatus::InSpinClockwise(x)
                        }
                        RotationStatus::NotInSpin => {}
                    }
                }

                PhysicsChangeType::AutoAccelerate => {
                    self.player_inputs.engine_status = EngineStatus::Accelerating(1.0);
                }

                PhysicsChangeType::TurnOnlyWhenNotMoving => {
                    if self.velocity != DVec3::ZERO {
                        self.player_inputs.rotation_status = RotationStatus::NotInSpin;
                    }
                }
                _ => (),
            }
        }
    }

    pub fn apply_physics_changes(&mut self) {
        for change in &self.physics_changes {
            match change.change_type {
                PhysicsChangeType::IAmSpeed => {
                    let flat_speed_increase = 30.0;
                    self.velocity = self.velocity * (self.velocity.length() + flat_speed_increase);
                }

                PhysicsChangeType::ShoppingCart => {
                    self.angular_velocity += self.stat(Stat::CarSpin) / 2.0;
                }

                PhysicsChangeType::InSpainButTheAIsSilent => {
                    match self.player_inputs.rotation_status {
                        RotationStatus::InSpinClockwise { .. } => {}
                        RotationStatus::NotInSpin => {
                            self.player_inputs.rotation_status =
                                RotationStatus::InSpinClockwise(1.0);
                            self.angular_velocity += self.stat(Stat::CarSpin);
                        }
                        RotationStatus::InSpinCounterclockwise(modifier) => {
                            self.player_inputs.rotation_status =
                                RotationStatus::InSpinClockwise(modifier);
                            self.angular_velocity += 2.0 * self.stat(Stat::CarSpin);
                        }
                    }
                }

                _ => {}
            }
        }
    }
}
