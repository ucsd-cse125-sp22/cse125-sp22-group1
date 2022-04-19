use chariot_core::physics_changes::PhysicsChangeType;
use glam::DVec3;

use chariot_core::entity_location::EntityLocation;
use chariot_core::player_inputs::EngineStatus;
use chariot_core::player_inputs::PlayerInputs;
use chariot_core::player_inputs::RotationStatus;
use chariot_core::GLOBAL_CONFIG;

mod collisions;
pub mod player_entity;

use player_entity::PlayerEntity;

impl PlayerEntity {
    /* Given a set of physical properties, compute and return what next tick's
    	* physics properties will be for that object */
    pub fn do_physics_step(
        &self,
        time_step: f64,
        mut potential_colliders: Vec<&PlayerEntity>,
    ) -> PlayerEntity {
        let self_forces = self.sum_of_self_forces();
        let acceleration = self_forces / self.mass;

        let angular_velocity: f64 = match self.player_inputs.rotation_status {
            RotationStatus::InSpinClockwise => self.angular_velocity + GLOBAL_CONFIG.car_spin,
            RotationStatus::InSpinCounterclockwise => {
                self.angular_velocity - GLOBAL_CONFIG.car_spin
            }
            RotationStatus::NotInSpin => {
                self.angular_velocity * GLOBAL_CONFIG.rotation_reduction_coefficient
            }
        };

        let mut delta_velocity = acceleration * time_step;

        for collider in potential_colliders.iter_mut() {
            delta_velocity += self.delta_v_from_collision_with_player(collider);
        }

        let mut new_player = PlayerEntity {
            player_inputs: PlayerInputs {
                engine_status: self.player_inputs.engine_status,
                rotation_status: self.player_inputs.rotation_status,
            },

            entity_location: EntityLocation {
                position: self.entity_location.position + self.velocity * time_step,
                unit_steer_direction: self.entity_location.unit_steer_direction,
            },

            velocity: self.velocity + delta_velocity,
            angular_velocity: angular_velocity,
            mass: self.mass,
            size: self.size,
            bounding_box: self.bounding_box,
            physics_changes: self.physics_changes.clone(),
        };

        new_player.apply_physics_changes();

        return new_player;
    }

    fn sum_of_self_forces(&self) -> DVec3 {
        return self.gravitational_force_on_object()
            + self.normal_force_on_object()
            + self.player_applied_force_on_object()
            + self.air_resistance_force_on_object()
            + self.rolling_resistance_force_on_object();
    }

    fn gravitational_force_on_object(&self) -> DVec3 {
        return DVec3::new(0.0, -1.0, 0.0) * self.mass * GLOBAL_CONFIG.gravity_coefficient;
    }

    // unjustified temporary assumption we'll invalidate later: we're always on flat
    // ground (otherwise, there's a horizontal component to normal force)
    fn normal_force_on_object(&self) -> DVec3 {
        return DVec3::new(0.0, 1.0, 0.0) * self.mass;
    }

    // Includes two player-applied forces: accelerator and brake.
    fn player_applied_force_on_object(&self) -> DVec3 {
        match self.player_inputs.engine_status {
            // The accelerator always applies forces in the steer direction; since
            // rotation is free, this is the intuitive direction. Braking, however,
            // is directionless, so the force of braking applies in a different
            // direction: specifically, it acts against whatever the current
            // direction of travel is. (which is not the steering direction!)
            EngineStatus::Accelerating => {
                return self.entity_location.unit_steer_direction
                    * self.mass
                    * GLOBAL_CONFIG.car_accelerator
            }
            // divide velocity by its magnitude to have a unit vector pointing
            // towards current heading, then apply the force in the reverse direction
            EngineStatus::Braking => {
                return self.velocity / self.velocity.length()
                    * -1.0
                    * self.mass
                    * GLOBAL_CONFIG.car_brake
            }
            // And there is no player-applied force when not accelerating or braking
            EngineStatus::Neutral => return DVec3::new(0.0, 0.0, 0.0),
        }
    }

    // Equations for modelling air resistance and rolling resistance come from
    // https://asawicki.info/Mirror/Car%20Physics%20for%20Games/Car%20Physics%20for%20Games.html
    fn air_resistance_force_on_object(&self) -> DVec3 {
        // air resistance is proportional to the square of velocity
        return self.velocity
            * self.mass
            * -1.0
            * GLOBAL_CONFIG.drag_coefficient
            * self.velocity.length();
    }

    fn rolling_resistance_force_on_object(&self) -> DVec3 {
        return self.velocity * self.mass * -1.0 * GLOBAL_CONFIG.rolling_resistance_coefficient;
    }

    fn apply_physics_changes(&mut self) {
        for change in &self.physics_changes {
            match change.change_type {
                PhysicsChangeType::IAmSpeed => {
                    let flat_speed_increase = 30.0;
                    self.velocity = self.velocity * (self.velocity.length() + flat_speed_increase);
                }
                PhysicsChangeType::NoTurningRight => {
                    if matches!(
                        self.player_inputs.rotation_status,
                        RotationStatus::InSpinClockwise
                    ) {
                        self.player_inputs.rotation_status = RotationStatus::NotInSpin;
                        self.angular_velocity -= GLOBAL_CONFIG.car_spin;
                    }
                }
                PhysicsChangeType::ShoppingCart => {
                    self.angular_velocity += GLOBAL_CONFIG.car_spin / 2.0;
                }
                PhysicsChangeType::InSpainButTheAIsSilent => {
                    match self.player_inputs.rotation_status {
                        RotationStatus::InSpinClockwise => {}
                        RotationStatus::NotInSpin => {
                            self.player_inputs.rotation_status = RotationStatus::InSpinClockwise;
                            self.angular_velocity += GLOBAL_CONFIG.car_spin;
                        }
                        RotationStatus::InSpinCounterclockwise => {
                            self.player_inputs.rotation_status = RotationStatus::InSpinClockwise;
                            self.angular_velocity += 2.0 * GLOBAL_CONFIG.car_spin;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::DVec3;

    use chariot_core::entity_location::EntityLocation;
    use chariot_core::player_inputs::EngineStatus;
    use chariot_core::player_inputs::PlayerInputs;
    use chariot_core::player_inputs::RotationStatus;
    use chariot_core::GLOBAL_CONFIG;

    use crate::physics::player_entity::PlayerEntity;

    #[test]
    fn test_accelerating() {
        let mut props = PlayerEntity {
            player_inputs: PlayerInputs {
                engine_status: EngineStatus::Accelerating,
                rotation_status: RotationStatus::NotInSpin,
            },

            entity_location: EntityLocation {
                position: DVec3::new(20.0, 30.0, 40.0),
                unit_steer_direction: DVec3::new(0.6, 0.0, 0.8),
            },

            velocity: DVec3::new(2.0, 0.0, 1.0),
            angular_velocity: 0.0,
            mass: 10.0,

            size: DVec3::new(10.0, 10.0, 10.0),
            bounding_box: [[-5.0, 5.0], [-5.0, 5.0], [-5.0, 5.0]],
            physics_changes: Vec::new(),
        };

        props = props.do_physics_step(1.0, Vec::new());

        // since we're accelerating, should have the following changes:
        // - should have moved forward by previous velocity times time step
        assert!(props
            .entity_location
            .position
            .abs_diff_eq(DVec3::new(22.0, 30.0, 41.0), 0.001));
        // - velocity should have increased by acceleration amount in steer
        // direction, and decreased because of drag and rolling resistance
        let expected_velocity = DVec3::new(2.0, 0.0, 1.0)
            + DVec3::new(0.6, 0.0, 0.8) * GLOBAL_CONFIG.car_accelerator
            + DVec3::new(-2.0, 0.0, -1.0) * GLOBAL_CONFIG.drag_coefficient * (5.0 as f64).sqrt()
            + DVec3::new(-2.0, 0.0, -1.0) * GLOBAL_CONFIG.rolling_resistance_coefficient;
        assert!(props.velocity.abs_diff_eq(expected_velocity, 0.001));
    }

    #[test]
    fn test_non_accelerating() {
        let mut props = PlayerEntity {
            player_inputs: PlayerInputs {
                engine_status: EngineStatus::Neutral,
                rotation_status: RotationStatus::NotInSpin,
            },

            entity_location: EntityLocation {
                position: DVec3::new(20.0, 30.0, 40.0),
                unit_steer_direction: DVec3::new(0.6, 0.0, 0.8),
            },

            velocity: DVec3::new(2.0, 0.0, 1.0),
            angular_velocity: 0.0,
            mass: 10.0,

            size: DVec3::new(10.0, 10.0, 10.0),
            bounding_box: [[15.0, 25.0], [25.0, 35.0], [35.0, 45.0]],
            physics_changes: Vec::new(),
        };

        props = props.do_physics_step(1.0, Vec::new());

        // since we're not accelerating, should have the following changes:
        // - should have moved forward by previous velocity times time step
        assert!(props
            .entity_location
            .position
            .abs_diff_eq(DVec3::new(22.0, 30.0, 41.0), 0.001));
        // - velocity should only have decreased, due to drag and rolling resistance
        let expected_velocity = DVec3::new(2.0, 0.0, 1.0)
            + DVec3::new(-2.0, 0.0, -1.0) * GLOBAL_CONFIG.drag_coefficient * (5.0 as f64).sqrt()
            + DVec3::new(-2.0, 0.0, -1.0) * GLOBAL_CONFIG.rolling_resistance_coefficient;
        assert!(props.velocity.abs_diff_eq(expected_velocity, 0.001));
    }
    #[test]
    fn test_decelerating() {
        let mut props = PlayerEntity {
            player_inputs: PlayerInputs {
                engine_status: EngineStatus::Braking,
                rotation_status: RotationStatus::NotInSpin,
            },

            entity_location: EntityLocation {
                position: DVec3::new(20.0, 30.0, 40.0),
                unit_steer_direction: DVec3::new(0.6, 0.0, 0.8),
            },

            velocity: DVec3::new(2.0, 0.0, 1.0),
            angular_velocity: 0.0,
            mass: 10.0,

            size: DVec3::new(10.0, 10.0, 10.0),
            bounding_box: [[15.0, 25.0], [25.0, 35.0], [35.0, 45.0]],
            physics_changes: Vec::new(),
        };

        props = props.do_physics_step(1.0, Vec::new());

        // since we're decelerating, should have the following changes:
        // - should have moved forward by previous velocity times time step
        assert!(props
            .entity_location
            .position
            .abs_diff_eq(DVec3::new(22.0, 30.0, 41.0), 0.001));
        // - velocity should only have decreased, due to braking, drag, and rolling resistance
        let prev_velocity = DVec3::new(2.0, 0.0, 1.0);
        let neg_prev_velocity = DVec3::new(-2.0, 0.0, -1.0);
        let expected_velocity = prev_velocity
            + (neg_prev_velocity / neg_prev_velocity.length()) * GLOBAL_CONFIG.car_brake
            + neg_prev_velocity * GLOBAL_CONFIG.drag_coefficient * (5.0 as f64).sqrt()
            + neg_prev_velocity * GLOBAL_CONFIG.rolling_resistance_coefficient;
        assert!(props.velocity.abs_diff_eq(expected_velocity, 0.001));
    }

    #[test]
    fn test_spinning() {
        let mut props = PlayerEntity {
            player_inputs: PlayerInputs {
                engine_status: EngineStatus::Braking,
                rotation_status: RotationStatus::InSpinClockwise,
            },

            entity_location: EntityLocation {
                position: DVec3::new(20.0, 30.0, 40.0),
                unit_steer_direction: DVec3::new(0.6, 0.0, 0.8),
            },

            velocity: DVec3::new(0.0, 0.0, 0.0),
            angular_velocity: 0.0,
            mass: 10.0,

            size: DVec3::new(10.0, 10.0, 10.0),
            bounding_box: [[15.0, 25.0], [25.0, 35.0], [35.0, 45.0]],
            physics_changes: Vec::new(),
        };

        props = props.do_physics_step(1.0, Vec::new());

        assert_eq!(props.angular_velocity, GLOBAL_CONFIG.car_spin);

        props.player_inputs.rotation_status = RotationStatus::NotInSpin;
        props = props.do_physics_step(1.0, Vec::new());

        assert_eq!(
            props.angular_velocity,
            GLOBAL_CONFIG.car_spin * GLOBAL_CONFIG.rotation_reduction_coefficient
        );
    }
}
