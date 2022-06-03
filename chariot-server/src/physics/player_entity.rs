use crate::game::powerup::PowerUp;
use crate::physics::bounding_box::BoundingBox;
use chariot_core::entity_location::EntityLocation;
use chariot_core::player::choices::{Chair, Stat};
use chariot_core::player::{
    lap_info::LapInformation,
    player_inputs::{EngineStatus, PlayerInputs, RotationStatus},
};
use chariot_core::sound_effect::SoundEffect;
use chariot_core::GLOBAL_CONFIG;
use glam::{DMat3, DVec3};

use crate::physics::trigger_entity::TriggerEntity;
use crate::progress::PlayerProgress;

use super::physics_changes::PhysicsChange;
use super::ramp::RampCollisionResult;
use super::stats_changes::StatsChange;

pub struct PlayerEntity {
    pub velocity: DVec3,
    pub angular_velocity: f64, // in radians per time unit

    pub size: DVec3,
    pub bounding_box: BoundingBox,

    pub player_inputs: PlayerInputs,
    pub entity_location: EntityLocation,

    pub current_colliders: Vec<BoundingBox>,

    pub physics_changes: Vec<PhysicsChange>,
    pub stats_changes: Vec<StatsChange>,

    pub sound_effects: Vec<SoundEffect>,

    pub placement_data: PlayerProgress,

    pub current_powerup: Option<PowerUp>,
    pub chair: Chair,
}

impl PlayerEntity {
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
        let mut modifier = 1.0;
        for change in &self.stats_changes {
            if change.stat == name {
                modifier *= change.multiplier;
            }
        }
        modifier
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
        speedup_zones: &Vec<BoundingBox>,
        ramp_collision_result: &RampCollisionResult,
    ) -> PlayerEntity {
        let mut has_collided_with_players = false;
        let minimum_player_height = match ramp_collision_result {
            RampCollisionResult::NoEffect => 1.0,
            RampCollisionResult::Collision { .. } => 1.0,
            RampCollisionResult::Driveable { ramp } => ramp.get_height_at_coordinates(
                self.entity_location.position.x,
                self.entity_location.position.z,
            ),
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

        let rotation_matrix = DMat3::from_axis_angle(DVec3::Y, -1.0 * angular_velocity);

        let mut delta_velocity = acceleration * time_step;

        for collider in potential_colliders.iter() {
            let delta_v = self.delta_v_from_collision_with_player(collider);
            delta_velocity += self.stat(Stat::PlayerBounciness) * delta_v;
            if delta_v != DVec3::ZERO {
                has_collided_with_players = true;
            }
        }

        let mut terrain_with_collisions = potential_terrain.clone();
        terrain_with_collisions.retain(|terrain| self.bounding_box.is_colliding(terrain));
        if let RampCollisionResult::Collision { ramp } = ramp_collision_result {
            terrain_with_collisions.push(ramp.bounding_box());
        }
        let collision_terrain_is_new = terrain_with_collisions != self.current_colliders;

        // Make sure we aren't too fast/slow, but BEFORE we bounce off walls or accelerate off ramps(which can be fast intentionally)
        // 1. velocity changes from bouncing off walls
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

        // 2. velocity changes from ramp-zooming
        if let RampCollisionResult::Driveable { ramp } = ramp_collision_result {
            if !self.is_aerial(ramp_collision_result) {
                let ramp_incline = ramp.get_incline_vector().normalize();

                // vroom vroom vroom
                new_velocity += ramp_incline;
            }
        }

        // We only react to colliding with a set of objects if we aren't already
        // colliding with them (otherwise, it's super easy to get stuck inside
        // an object)
        if collision_terrain_is_new {
            let multiplier = -(1.0 + self.stat(Stat::TerrainBounciness));
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

        // If not in contact with any speedup zones (= in the air or off-track), apply a speed penalty
        if !self.is_aerial(ramp_collision_result)
            && speedup_zones
                .iter()
                .all(|zone| !zone.is_colliding(&self.bounding_box))
        {
            new_velocity *= 1.0 - GLOBAL_CONFIG.off_track_speed_penalty;
        }

        let new_steer_direction =
            rotation_matrix * self.entity_location.unit_steer_direction.normalize();

        let mut new_position = self.entity_location.position + self.velocity * time_step;
        if new_position.y < minimum_player_height {
            new_position.y = minimum_player_height;
        }

        let mut sound_effects = vec![];

        if collision_terrain_is_new {
            sound_effects.push(SoundEffect::TerrainCollision);
        }
        if has_collided_with_players {
            sound_effects.push(SoundEffect::PlayerCollision);
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
            stats_changes: self.stats_changes.clone(),
            sound_effects,
            lap_info: self.lap_info,
            current_powerup: self.current_powerup,
            chair: self.chair,
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

    fn is_aerial(&self, ramp_collision_result: &RampCollisionResult) -> bool {
        let ground_level = match ramp_collision_result {
            RampCollisionResult::NoEffect => 0.0,
            RampCollisionResult::Collision { ramp } | RampCollisionResult::Driveable { ramp } => {
                ramp.get_height_at_coordinates(
                    self.entity_location.position.x,
                    self.entity_location.position.z,
                )
            }
        };

        self.entity_location.position[1] - 1.0 > ground_level
    }

    fn sum_of_self_forces(&self, ramp_collision_result: &RampCollisionResult) -> DVec3 {
        let gravitational_force = self.gravitational_force_on_object();
        let air_forces = gravitational_force
            + self.player_applied_force_on_object()
            + self.air_resistance_force_on_object();

        let mut normal_force = self.normal_force_on_object();
        if normal_force.length_squared() > gravitational_force.length_squared() {
            normal_force = normal_force * gravitational_force.length() / normal_force.length();
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
}
