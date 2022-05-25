use std::collections::HashMap;

use crate::game::powerup::PowerUp;
use crate::physics::bounding_box::BoundingBox;
use chariot_core::entity_location::EntityLocation;
use chariot_core::player::choices::{Chair, Stat};
use chariot_core::player::{
    lap_info::LapInformation,
    physics_changes::{PhysicsChange, PhysicsChangeType},
    player_inputs::{EngineStatus, PlayerInputs, RotationStatus},
};
use chariot_core::GLOBAL_CONFIG;
use glam::{DMat3, DQuat, DVec2, DVec3};

use crate::physics::trigger_entity::TriggerEntity;

use super::ramp::{Ramp, RampCollisionResult};

fn get_index_of_ramp_with_potential_effect(x: f64, z: f64, ramps: &Vec<Ramp>) -> Option<usize> {
    let ramp_heights: Vec<usize> = ramps
        .iter()
        .enumerate()
        .filter(|(_, ramp)| ramp.coordinates_in_footprint(x, z))
        .map(|(index, _)| index)
        .collect();

    if ramp_heights.len() > 0 {
        Some(*ramp_heights.get(0).unwrap())
    } else {
        None
    }
}

pub struct PlayerEntity {
    pub velocity: DVec3,
    pub angular_velocity: f64, // in radians per time unit

    pub size: DVec3,
    pub bounding_box: BoundingBox,

    pub player_inputs: PlayerInputs,
    pub entity_location: EntityLocation,

    pub current_colliders: Vec<BoundingBox>,

    pub physics_changes: Vec<PhysicsChange>,

    pub lap_info: LapInformation,

    pub current_powerup: Option<PowerUp>,
    pub chair: Chair,
    pub stat_modifiers: HashMap<Stat, f64>,
}

impl PlayerEntity {
    // Get upward direction based on position of wheels on this ramp; if it's too steep to traverse, return None instead of the new upward angle
    fn get_upward_direction_on_ramp(&self, ramp: &Ramp) -> DVec3 {
        let BoundingBox {
            min_x,
            max_x,
            min_z,
            max_z,
            ..
        } = self.bounding_box;
        let ll_height = ramp.get_height_at_coordinates(min_x, min_z);
        let lr_height = ramp.get_height_at_coordinates(max_x, min_z);
        let ul_height = ramp.get_height_at_coordinates(min_x, max_z);
        let ur_height = ramp.get_height_at_coordinates(max_x, max_z);
        let lower_left_corner = DVec3::new(min_x, ll_height, min_z);
        let lower_right_corner = DVec3::new(max_x, lr_height, min_z);
        let upper_left_corner = DVec3::new(min_x, ul_height, max_z);
        let upper_right_corner = DVec3::new(max_x, ur_height, max_z);

        let diagonal_1 = lower_right_corner - upper_left_corner;
        let diagonal_2 = upper_right_corner - lower_left_corner;

        let mut upward = diagonal_2.cross(diagonal_1);
        // when close, these can oscillate back and forth, so just make sure it's pointing positive
        if upward.y < 0.0 {
            upward *= -1.0;
        }
        upward.normalize()
    }

    // whenever not on a ramp, we flatten out, instead of being all wonky
    fn get_upward_direction_off_ramp(&self) -> DVec3 {
        let upward = self.entity_location.unit_upward_direction;
        if upward != DVec3::Y {
            // this is normal to the plane which contains the Y-axis and the
            // old upward direction; when upward_direction is equal to Y, the
            // cross product is zero, so skip that possibility
            let rotation_axis = DVec3::Y.cross(upward);
            let rotation_matrix = DQuat::from_axis_angle(rotation_axis, -0.1);

            return (rotation_matrix * upward).normalize();
        } else {
            return DVec3::Y;
        }
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

    // given a player and a ramp the player is potentially colliding with,
    // return whether the player is allowed on  (not allowed on => collision)
    pub fn is_allowed_onto_ramp(&self, ramp: &Ramp) -> bool {
        let [[ramp_min_x, ramp_max_x], [ramp_min_z, ramp_max_z]] = ramp.footprint;
        let x = self.entity_location.position.x;
        let z = self.entity_location.position.z;
        let x_vel = self.velocity.x;
        let z_vel = self.velocity.z;

        // let a player onto the ramp if they're either within the footprint of
        // the ramp, or in the strip leading out from the ramp face
        if x >= ramp_min_x && x <= ramp_max_x && z >= ramp_min_z && z <= ramp_max_z {
            return ramp.get_height_at_coordinates(x, z)
                - (self.entity_location.position.y - self.size[1] / 2.0)
                <= 1.0;
        }

        // to enter the ramp, you must be within the strip leading out from the
        // low side of the ramp, and have velocity that's pointed along the
        // incline direction
        if ramp.incline_direction == DVec2::X {
            // e.g. if the ramp inclines in the direction of positive x, let em
            // on if their z is bounded by the ramp's and their x isn't past
            // ramp's largest x
            z >= ramp_min_z && z <= ramp_max_z && x <= ramp_max_x && x_vel > 0.0
        } else if ramp.incline_direction == -1.0 * DVec2::X {
            z >= ramp_min_z && z <= ramp_max_z && x >= ramp_min_x && x_vel < 0.0
        } else if ramp.incline_direction == DVec2::Y {
            // Y means Z :3
            x >= ramp_min_x && x <= ramp_max_x && z <= ramp_max_z && z_vel > 0.0
        } else if ramp.incline_direction == -1.0 * DVec2::Y {
            x >= ramp_min_x && x <= ramp_max_x && z >= ramp_min_z && z_vel < 0.0
        } else {
            // figure this out if we ever get non-orthogonal incline directions (unlikely)
            false
        }
    }

    pub fn update_upwards_from_ramps(
        &mut self,
        potential_ramps: &Vec<Ramp>,
    ) -> Option<RampCollisionResult> {
        let BoundingBox {
            min_x,
            max_x,
            min_z,
            max_z,
            ..
        } = self.bounding_box;

        let index_of_ramp_with_effect = [
            [min_x, min_z],
            [min_x, max_z],
            [max_x, min_z],
            [max_x, max_z],
        ]
        .iter()
        .map(|[x, z]| get_index_of_ramp_with_potential_effect(*x, *z, &potential_ramps))
        .fold(None, |acc, e| if e.is_some() { e } else { acc });

        match index_of_ramp_with_effect {
            Some(index) => {
                let ramp = potential_ramps.get(index).unwrap().clone();
                let can_get_on = self.is_allowed_onto_ramp(&ramp);

                if can_get_on {
                    let upward = self.get_upward_direction_on_ramp(&ramp);
                    self.entity_location.unit_upward_direction = upward;
                }
                Some(RampCollisionResult { ramp, can_get_on })
            }
            None => {
                self.entity_location.unit_upward_direction = self.get_upward_direction_off_ramp();
                // return None; we don't have anything to do with any ramps
                None
            }
        }
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
        let m1 = self.stat(Stat::Mass);
        let m2 = self.stat(Stat::Mass);
        let x1 = self.entity_location.position;
        let x2 = other.entity_location.position;

        let term1 = (2.0 * m2) / (m1 + m2);
        let term2 = (v1 - v2).dot(x1 - x2) / (x1 - x2).length_squared();
        let term3 = x1 - x2;

        let result = term1 * term2 * term3;
        return DVec3::new(result.x, 0.0, result.z);
    }

    pub fn get_stat_modifier(&self, name: Stat) -> f64 {
        *self.stat_modifiers.get(&name).unwrap_or(&1.0)
    }

    pub fn _mod_stat_modifier(&mut self, name: Stat, value: f64) {
        self.stat_modifiers
            .insert(name, self.get_stat_modifier(name) * value);
    }

    pub fn _reset_stat_modifier(&mut self, name: Stat) {
        self.stat_modifiers.remove(&name);
    }

    pub fn stat(&self, name: Stat) -> f64 {
        self.get_stat_modifier(name) * self.chair.stat(&name)
    }

    /* Given a set of physical properties, compute and return what next tick's
     * physics properties will be for that object */
    pub fn do_physics_step<'a>(
        &self,
        time_step: f64,
        potential_colliders: Vec<&PlayerEntity>,
        potential_terrain: Vec<BoundingBox>,
        potential_triggers: impl Iterator<Item = &'a mut dyn TriggerEntity>,
        ramp_collision_result: Option<&RampCollisionResult>,
    ) -> PlayerEntity {
        let minimum_player_height =
            if ramp_collision_result.is_some() && ramp_collision_result.unwrap().can_get_on {
                ramp_collision_result
                    .unwrap()
                    .ramp
                    .get_height_at_coordinates(
                        self.entity_location.position.x,
                        self.entity_location.position.z,
                    )
                    + 1.0
            } else {
                1.0
            };
        let self_forces = self.sum_of_self_forces(ramp_collision_result);
        let acceleration = self_forces / self.stat(Stat::Mass);

        let angular_velocity: f64 = match self.player_inputs.rotation_status {
            RotationStatus::InSpinClockwise(modifier) => f64::min(
                modifier as f64 * self.stat(Stat::MaxCarSpin),
                self.angular_velocity + modifier as f64 * self.stat(Stat::CarSpin),
            ),
            RotationStatus::InSpinCounterclockwise(modifier) => f64::max(
                -modifier as f64 * self.stat(Stat::MaxCarSpin),
                self.angular_velocity - modifier as f64 * self.stat(Stat::CarSpin),
            ),
            RotationStatus::NotInSpin => {
                self.angular_velocity * self.stat(Stat::RotationReductionCoefficient)
            }
        };

        let rotation_matrix = glam::DMat3::from_axis_angle(DVec3::Y, -1.0 * angular_velocity);

        let mut delta_velocity = acceleration * time_step;

        for collider in potential_colliders.iter() {
            delta_velocity += self.delta_v_from_collision_with_player(collider);
        }

        let mut terrain_with_collisions = potential_terrain.clone();
        terrain_with_collisions.retain(|terrain| self.bounding_box.is_colliding(terrain));
        if ramp_collision_result.is_some() && !ramp_collision_result.unwrap().can_get_on {
            terrain_with_collisions.push(ramp_collision_result.unwrap().ramp.bounding_box());
        }
        let collision_terrain_is_new = terrain_with_collisions != self.current_colliders;

        // Make sure we aren't too fast/slow, but BEFORE we bounce off walls (which can be fast intentionally)
        let mut new_velocity = self.velocity + delta_velocity;
        if new_velocity.length() > self.stat(Stat::MaxCarSpeed) {
            new_velocity = new_velocity.normalize() * self.stat(Stat::MaxCarSpeed);
        } else if new_velocity.length() < 0.0005 {
            new_velocity = DVec3::ZERO;
        } else if new_velocity.dot(self.velocity) < 0.0 {
            // If we are trying to reverse direction and are braking, we should just stop isntead
            if let EngineStatus::Braking = self.player_inputs.engine_status {
                new_velocity = DVec3::ZERO;
            }
        }

        // We only react to colliding with a set of objects if we aren't already
        // colliding with them (otherwise, it's super easy to get stuck inside
        // an object)
        if collision_terrain_is_new {
            let multiplier = -(1.0 + GLOBAL_CONFIG.wall_bounciness);
            for terrain in &terrain_with_collisions {
                // We want to "reflect" off of objects: this means negating the
                // x component of velocity if hitting a face parallel to the
                // z-axis, and vice versa. But if we're already going away from
                // an object, we don't want to change that direction of
                // velocity.
                let x = self.entity_location.position.x;
                let z = self.entity_location.position.z;

                if x >= terrain.min_x && x <= terrain.max_x {
                    if (z < terrain.max_z && self.velocity.z > 0.0)
                        || (z > terrain.min_z && self.velocity.z < 0.0)
                    {
                        new_velocity.z += multiplier * self.velocity.z;
                    }
                } else {
                    if (x < terrain.max_x && self.velocity.x > 0.0)
                        || (x > terrain.min_x && self.velocity.x < 0.0)
                    {
                        new_velocity.x += multiplier * self.velocity.x;
                    }
                }
            }
        }

        let new_steer_direction =
            rotation_matrix * self.entity_location.unit_steer_direction.normalize();

        let mut new_position = self.entity_location.position + self.velocity * time_step;
        if new_position.y < minimum_player_height {
            new_position.y = minimum_player_height;
        }

        let mut new_player = PlayerEntity {
            player_inputs: PlayerInputs {
                engine_status: self.player_inputs.engine_status,
                rotation_status: self.player_inputs.rotation_status,
            },

            entity_location: EntityLocation {
                position: new_position,
                unit_steer_direction: new_steer_direction,
                unit_upward_direction: self.entity_location.unit_upward_direction,
            },

            current_colliders: terrain_with_collisions,

            velocity: new_velocity,
            angular_velocity,
            size: self.size,
            bounding_box: self.bounding_box,
            physics_changes: self.physics_changes.clone(),
            lap_info: self.lap_info,
            current_powerup: self.current_powerup,
            chair: self.chair,
            stat_modifiers: self.stat_modifiers.to_owned(),
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

    fn is_aerial(&self, ramp_collision_result: Option<&RampCollisionResult>) -> bool {
        let ground_level = if ramp_collision_result.is_some() {
            ramp_collision_result
                .unwrap()
                .ramp
                .get_height_at_coordinates(
                    self.entity_location.position[0],
                    self.entity_location.position[2],
                )
        } else {
            0.0
        };

        self.entity_location.position[1] - (self.size[1] / 2.0) > ground_level
    }

    fn sum_of_self_forces(&self, ramp_collision_result: Option<&RampCollisionResult>) -> DVec3 {
        let gravitational_force = self.gravitational_force_on_object();
        let mut air_forces = gravitational_force
            + self.player_applied_force_on_object()
            + self.air_resistance_force_on_object();

        let mut normal_force = self.normal_force_on_object();
        if normal_force.length_squared() > gravitational_force.length_squared() {
            normal_force = normal_force * gravitational_force.length() / normal_force.length();
        }

        if ramp_collision_result.is_some() && ramp_collision_result.unwrap().can_get_on {
            // vroom vroom vroom
            air_forces += self.acceleration_force_on_object() * 20.0;
        }

        return if self.is_aerial(ramp_collision_result) {
            air_forces
        } else {
            air_forces + normal_force + self.rolling_resistance_force_on_object()
        };
    }

    fn gravitational_force_on_object(&self) -> DVec3 {
        return DVec3::new(0.0, -1.0, 0.0)
            * self.stat(Stat::Mass)
            * self.stat(Stat::GravityCoefficient);
    }

    fn normal_force_on_object(&self) -> DVec3 {
        return self.entity_location.unit_upward_direction * self.stat(Stat::Mass);
    }

    fn acceleration_force_on_object(&self) -> DVec3 {
        let (up_x, up_y, up_z) = self.entity_location.unit_upward_direction.into();
        let pitch = DVec3::new(up_x, up_y, 0.0).angle_between(DVec3::Y);
        let roll = DVec3::new(0.0, up_y, up_z).angle_between(DVec3::Y);
        let pitch_rotation_matrix = DMat3::from_rotation_z(pitch);
        let roll_rotation_matrix = DMat3::from_rotation_x(roll);

        let acceleration_direction = pitch_rotation_matrix
            .mul_vec3(roll_rotation_matrix.mul_vec3(self.entity_location.unit_steer_direction));

        acceleration_direction.normalize() * self.stat(Stat::Mass) * self.stat(Stat::CarAccelerator)
    }

    // Includes two player-applied forces: accelerator and brake.
    fn player_applied_force_on_object(&self) -> DVec3 {
        match self.player_inputs.engine_status {
            // The accelerator always applies forces in the steer direction; since
            // rotation is free, this is the intuitive direction. Braking, however,
            // is directionless, so the force of braking applies in a different
            // direction: specifically, it acts against whatever the current
            // direction of travel is. (which is not the steering direction!)
            EngineStatus::Accelerating(_modifier) => self.acceleration_force_on_object(),
            // apply the force in the reverse direction of current velocity;
            // just do nothing if velocity is zero
            EngineStatus::Braking => {
                return self.velocity.normalize_or_zero()
                    * -1.0
                    * self.stat(Stat::Mass)
                    * self.stat(Stat::CarBrake);
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
            * self.stat(Stat::Mass)
            * -1.0
            * self.stat(Stat::DragCoefficient)
            * self.velocity.length();
    }

    fn rolling_resistance_force_on_object(&self) -> DVec3 {
        return self.velocity
            * self.stat(Stat::Mass)
            * -1.0
            * self.stat(Stat::RollingResistanceCoefficient);
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
                        self.angular_velocity += self.stat(Stat::CarSpin);
                    }
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
            }
        }
    }
}
