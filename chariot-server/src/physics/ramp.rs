use glam::{DQuat, DVec2, DVec3};

use super::{bounding_box::BoundingBox, player_entity::PlayerEntity};

// Defines the 2D footprint of a collideable region:
// [[min_x, max_x]; [min_z, max_z]]
type Footprint = [[f64; 2]; 2];

#[derive(Clone, Copy, Debug)]
pub struct Ramp {
    pub footprint: Footprint,
    pub min_height: f64,
    pub max_height: f64,
    // points in the direction of the incline
    pub incline_direction: DVec2,
}

pub struct RampCollisionResult {
    pub ramp: Ramp,
    // true: can drive on top of the ramp, false: collides with the ramp and should bounce off
    pub can_get_on: bool,
}

impl Ramp {
    pub fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            min_x: self.footprint[0][0],
            max_x: self.footprint[0][1],
            min_y: self.min_height,
            max_y: self.max_height,
            min_z: self.footprint[1][0],
            max_z: self.footprint[1][1],
        }
    }

    pub fn coordinates_in_footprint(&self, x: f64, z: f64) -> bool {
        x >= self.footprint[0][0]
            && x <= self.footprint[0][1]
            && z >= self.footprint[1][0]
            && z <= self.footprint[1][1]
    }

    fn get_low_and_high_corners(&self) -> (DVec2, DVec2) {
        let [[min_x, max_x], [min_z, max_z]] = self.footprint;
        let incline_x = self.incline_direction.x;
        let incline_z = self.incline_direction.y;

        let lower_left = DVec2::new(min_x, min_z);
        let upper_left = DVec2::new(min_x, max_z);
        let lower_right = DVec2::new(max_x, min_z);
        let upper_right = DVec2::new(max_x, max_z);

        // top four cases are probably the only ones we need (incline is axis-orthogonal)
        if incline_x == 0.0 && incline_z > 0.0 {
            (lower_left, upper_left)
        } else if incline_x == 0.0 && incline_z < 0.0 {
            (upper_left, lower_left)
        } else if incline_x < 0.0 && incline_z == 0.0 {
            (lower_right, lower_left)
        } else if incline_x > 0.0 && incline_z == 0.0 {
            (lower_left, lower_right)
        // rest of these are mostly for completeness
        } else if incline_x > 0.0 && incline_z > 0.0 {
            (lower_left, upper_right)
        } else if incline_x > 0.0 && incline_z < 0.0 {
            (upper_left, lower_right)
        } else if incline_x < 0.0 && incline_z < 0.0 {
            (upper_right, lower_left)
        } else if incline_x < 0.0 && incline_z > 0.0 {
            (lower_right, upper_left)
        } else {
            (lower_left, upper_left)
        }
    }

    pub fn get_height_at_coordinates(&self, x: f64, z: f64) -> f64 {
        if !self.coordinates_in_footprint(x, z) {
            return 0.0;
        }

        let min_x = self.footprint[0][0];
        let min_z = self.footprint[1][0];
        let (low_corner, high_corner) = self.get_low_and_high_corners();

        let incline_vector = high_corner - low_corner;
        let ramp_height_proportion = DVec2::new(x - min_x, z - min_z)
            .project_onto(incline_vector)
            .length()
            / incline_vector.length();

        self.min_height + (ramp_height_proportion * (self.max_height - self.min_height))
    }

    pub fn get_incline_vector(&self) -> DVec3 {
        let (low_corner, high_corner) = self.get_low_and_high_corners();

        let incline_vec = DVec3::new(high_corner.x, self.max_height, high_corner.y)
            - DVec3::new(low_corner.x, self.min_height, low_corner.y);
        incline_vec
    }
}

impl PlayerEntity {
    fn get_index_of_ramp_with_potential_effect(&self, ramps: &Vec<Ramp>) -> Option<usize> {
        let ramp_heights: Vec<usize> = ramps
            .iter()
            .enumerate()
            .filter(|(_, ramp)| {
                ramp.coordinates_in_footprint(
                    self.entity_location.position.x,
                    self.entity_location.position.z,
                )
            })
            .map(|(index, _)| index)
            .collect();

        if ramp_heights.len() > 0 {
            Some(*ramp_heights.get(0).unwrap())
        } else {
            None
        }
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
            return ramp.get_height_at_coordinates(x, z) - (self.entity_location.position.y - 1.0)
                < 1.0;
        }

        // to enter the ramp, you must be within the strip leading out from the
        // low side of the ramp by 1 unit, and have velocity that's pointed along the
        // incline direction
        if ramp.incline_direction == DVec2::X {
            // e.g. if the ramp inclines in the direction of positive x, let em
            // on if their z is bounded by the ramp's and their x isn't past
            // ramp's largest x
            self.bounding_box.max_z >= ramp_min_z
                && self.bounding_box.min_z <= ramp_max_z
                && self.bounding_box.min_x <= ramp_max_x
                && self.bounding_box.min_x >= ramp_max_x - 1.0
                && x_vel > 0.0
        } else if ramp.incline_direction == -1.0 * DVec2::X {
            self.bounding_box.max_z >= ramp_min_z
                && self.bounding_box.min_z <= ramp_max_z
                && self.bounding_box.max_x >= ramp_min_x
                && self.bounding_box.max_x <= ramp_min_x + 1.0
                && x_vel < 0.0
        } else if ramp.incline_direction == DVec2::Y {
            // Y means Z :3
            self.bounding_box.max_x >= ramp_min_x
                && self.bounding_box.min_x <= ramp_max_x
                && self.bounding_box.min_z <= ramp_max_z
                && self.bounding_box.min_z >= ramp_max_z - 1.0
                && z_vel > 0.0
        } else if ramp.incline_direction == -1.0 * DVec2::Y {
            self.bounding_box.max_x >= ramp_min_x
                && self.bounding_box.min_x <= ramp_max_x
                && self.bounding_box.max_z >= ramp_min_z
                && self.bounding_box.max_z <= ramp_min_z + 1.0
                && z_vel < 0.0
        } else {
            // figure this out if we ever get non-orthogonal incline directions (unlikely)
            false
        }
    }

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

    pub fn update_upwards_from_ramps(
        &mut self,
        potential_ramps: &Vec<Ramp>,
    ) -> Option<RampCollisionResult> {
        match self.get_index_of_ramp_with_potential_effect(&potential_ramps) {
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
}
