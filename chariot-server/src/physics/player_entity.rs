use std::ops::Bound;

use crate::game::powerup::PowerUp;
use crate::physics::bounding_box::BoundingBox;
use chariot_core::entity_location::EntityLocation;
use chariot_core::player::{
    lap_info::LapInformation,
    physics_changes::{PhysicsChange, PhysicsChangeType},
    player_inputs::{EngineStatus, PlayerInputs, RotationStatus},
};
use chariot_core::GLOBAL_CONFIG;
use glam::DVec3;

use crate::physics::trigger_entity::TriggerEntity;

fn get_height_at_coordinates(_x: f64, _z: f64) -> f64 {
    return 0.0;
}

pub struct PlayerEntity {
    pub velocity: DVec3,
    pub angular_velocity: f64, // in radians per time unit

    pub mass: f64,
    pub size: DVec3,
    pub bounding_box: BoundingBox,

    pub player_inputs: PlayerInputs,
    pub entity_location: EntityLocation,

    pub current_colliders: Vec<BoundingBox>,

    pub physics_changes: Vec<PhysicsChange>,

    pub lap_info: LapInformation,

    pub current_powerup: Option<PowerUp>,
}

impl PlayerEntity {
    // set the upward direction based on the bounding box
    pub fn set_upward_direction_from_bounding_box(&mut self) {
        let BoundingBox {
            min_x,
            max_x,
            min_z,
            max_z,
            ..
        } = self.bounding_box;

        let lower_left_corner = DVec3::new(min_x, get_height_at_coordinates(min_x, min_z), min_z);
        let lower_right_corner = DVec3::new(max_x, get_height_at_coordinates(max_x, min_z), min_z);
        let upper_left_corner = DVec3::new(min_x, get_height_at_coordinates(min_x, max_z), max_z);
        let upper_right_corner = DVec3::new(max_x, get_height_at_coordinates(max_x, max_z), max_z);

        let diagonal_1 = lower_right_corner - upper_left_corner;
        let diagonal_2 = upper_right_corner - lower_left_corner;

        self.entity_location.unit_upward_direction = diagonal_2.cross(diagonal_1).normalize();
    }

    // update the underlying bounding box based on position, size, and steer angles
    pub fn update_bounding_box(&mut self) {
        // unit_steer_direction defines yaw, unit_upward_direction can be
        // decomposed into pitch and roll; with these, we can get Euler angles
        // for the 3d rotation

        let yaw = DVec3::new(
            self.entity_location.unit_steer_direction.x,
            0.0,
            self.entity_location.unit_steer_direction.z,
        )
        .angle_between(DVec3::X);
        let (up_x, up_y, up_z) = self.entity_location.unit_upward_direction.into();

        // positive on the x-axis is by default forward
        let pitch = DVec3::new(up_x, up_y, 0.0).angle_between(DVec3::Y);
        let roll = DVec3::new(0.0, up_y, up_z).angle_between(DVec3::Y);

        self.bounding_box.set_dimensions(
            &self.entity_location.position,
            &self.size,
            pitch,
            yaw,
            roll,
        );
    }

    // Returns the velocity change to self from colliding with other
    pub fn delta_v_from_collision_with_player(&self, other: &PlayerEntity) -> DVec3 {
        if !self.bounding_box.is_colliding(&other.bounding_box) {
            return DVec3::new(0.0, 0.0, 0.0);
        }

        // Uses the angle-free equation from
        // https://en.wikipedia.org/wiki/Elastic_collision#Two-dimensional
        // Which applies symmetrically so it shouldn't be much of a performance
        // hit to call this method once for each member of a colliding pair -
        // and the formula should be fast anyways.

        let v1 = self.velocity;
        let v2 = other.velocity;
        let m1 = self.mass;
        let m2 = other.mass;
        let x1 = self.entity_location.position;
        let x2 = other.entity_location.position;

        let term1 = (2.0 * m2) / (m1 + m2);
        let term2 = (v1 - v2).dot(x1 - x2) / (x1 - x2).length_squared();
        let term3 = x1 - x2;

        let result = term1 * term2 * term3;
        return DVec3::new(result.x, 0.0, result.z);
    }

    /* Given a set of physical properties, compute and return what next tick's
     * physics properties will be for that object */
    pub fn do_physics_step<'a>(
        &self,
        time_step: f64,
        potential_colliders: Vec<&PlayerEntity>,
        potential_terrain: Vec<BoundingBox>,
        potential_triggers: impl Iterator<Item = &'a mut dyn TriggerEntity>,
    ) -> PlayerEntity {
        let self_forces = self.sum_of_self_forces();
        let acceleration = self_forces / self.mass;

        let angular_velocity: f64 = match self.player_inputs.rotation_status {
            RotationStatus::InSpinClockwise => f64::min(
                GLOBAL_CONFIG.max_car_spin,
                self.angular_velocity + GLOBAL_CONFIG.car_spin,
            ),
            RotationStatus::InSpinCounterclockwise => f64::max(
                -GLOBAL_CONFIG.max_car_spin,
                self.angular_velocity - GLOBAL_CONFIG.car_spin,
            ),
            RotationStatus::NotInSpin => {
                self.angular_velocity * GLOBAL_CONFIG.rotation_reduction_coefficient
            }
        };

        let rotation_matrix = glam::Mat3::from_axis_angle(
            self.entity_location.unit_upward_direction.as_vec3(),
            -1.0 * angular_velocity as f32,
        );

        let mut delta_velocity = acceleration * time_step;

        for collider in potential_colliders.iter() {
            delta_velocity += self.delta_v_from_collision_with_player(collider);
        }

        let mut terrain_with_collisions = potential_terrain.clone();
        terrain_with_collisions.retain(|terrain| self.bounding_box.is_colliding(terrain));
        let collision_terrain_is_new = terrain_with_collisions != self.current_colliders;

        // We only react to colliding with a set of objects if we aren't already
        // colliding with them (otherwise, it's super easy to get stuck inside
        // an object)
        if collision_terrain_is_new {
            for terrain in &terrain_with_collisions {
                // We want to "reflect" off of objects: this means negating the
                // x component of velocity if hitting a face parallel to the
                // z-axis, and vice versa
                if self.entity_location.position.x >= terrain.min_x
                    && self.entity_location.position.x <= terrain.max_x
                {
                    delta_velocity.z += -2.0 * self.velocity.z;
                } else {
                    delta_velocity.x += -2.0 * self.velocity.x;
                }
            }
        }

        let mut new_velocity = self.velocity + delta_velocity;
        if new_velocity.length() > GLOBAL_CONFIG.max_car_speed {
            new_velocity = new_velocity.normalize() * GLOBAL_CONFIG.max_car_speed;
        } else if new_velocity.length() < 0.05 {
            new_velocity = DVec3::ZERO;
        }

        let new_steer_direction =
		// we want to instantly snap to the new direction if bouncing off an object (otherwise is confusing)
		if collision_terrain_is_new {
			new_velocity.normalize()
		} else {
			rotation_matrix
				.mul_vec3(self.entity_location.unit_steer_direction.as_vec3())
				.normalize()
				.as_dvec3()
		};

        let mut new_player = PlayerEntity {
            player_inputs: PlayerInputs {
                engine_status: self.player_inputs.engine_status,
                rotation_status: self.player_inputs.rotation_status,
            },

            entity_location: EntityLocation {
                position: self.entity_location.position + self.velocity * time_step,
                unit_steer_direction: if collision_terrain_is_new {
                    new_velocity.normalize()
                } else {
                    new_steer_direction
                },
                unit_upward_direction: self.entity_location.unit_upward_direction,
            },

            current_colliders: terrain_with_collisions,

            velocity: new_velocity,
            angular_velocity,
            mass: self.mass,
            size: self.size,
            bounding_box: self.bounding_box,
            physics_changes: self.physics_changes.clone(),
            lap_info: self.lap_info,
            current_powerup: None,
        };

        new_player.apply_physics_changes();

        for trigger in potential_triggers {
            if trigger
                .get_bounding_box()
                .is_colliding(&new_player.bounding_box)
            {
                trigger.trigger(&mut new_player);
            }
        }

        return new_player;
    }

    fn is_aerial(&self) -> bool {
        return self.entity_location.position[1]
            > self.size[1]
                + get_height_at_coordinates(
                    self.entity_location.position[0],
                    self.entity_location.position[2],
                );
    }

    fn sum_of_self_forces(&self) -> DVec3 {
        let air_forces = self.gravitational_force_on_object()
            + self.player_applied_force_on_object()
            + self.air_resistance_force_on_object();

        return if self.is_aerial() {
            air_forces
        } else {
            air_forces + self.normal_force_on_object() + self.rolling_resistance_force_on_object()
        };
    }

    fn gravitational_force_on_object(&self) -> DVec3 {
        return DVec3::new(0.0, -1.0, 0.0) * self.mass * GLOBAL_CONFIG.gravity_coefficient;
    }

    fn normal_force_on_object(&self) -> DVec3 {
        return self.entity_location.unit_upward_direction * self.mass;
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
                    * GLOBAL_CONFIG.car_accelerator;
            }
            // apply the force in the reverse direction of current velocity;
            // just do nothing if velocity is zero
            EngineStatus::Braking => {
                return self.velocity.normalize_or_zero()
                    * -1.0
                    * self.mass
                    * GLOBAL_CONFIG.car_brake;
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
                PhysicsChangeType::NoTurningLeft => {
                    if matches!(
                        self.player_inputs.rotation_status,
                        RotationStatus::InSpinCounterclockwise
                    ) {
                        self.player_inputs.rotation_status = RotationStatus::NotInSpin;
                        self.angular_velocity += GLOBAL_CONFIG.car_spin;
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
