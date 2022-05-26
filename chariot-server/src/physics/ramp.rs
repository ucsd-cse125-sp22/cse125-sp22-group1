use glam::{DVec2, DVec3};

use super::bounding_box::BoundingBox;

#[derive(Clone, Copy, Debug)]
pub struct Ramp {
    // [[min_x, max_x]; [min_z, max_z]]
    pub footprint: [[f64; 2]; 2],
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

    fn get_high_and_low_corner(&self) -> (DVec2, DVec2) {
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
        } else if incline_x > 0.0 && incline_z < 0.0 {
            (upper_left, lower_right)
        } else if incline_x > 0.0 && incline_z == 0.0 {
            (lower_left, lower_right)
        // rest of these are mostly for completeness
        } else if incline_x > 0.0 && incline_z > 0.0 {
            (lower_left, upper_right)
        } else if incline_x < 0.0 && incline_z < 0.0 {
            (upper_right, lower_left)
        } else if incline_x < 0.0 && incline_z == 0.0 {
            (lower_right, lower_left)
        } else if incline_x < 0.0 && incline_z > 0.0 {
            (lower_right, upper_left)
        } else {
            (lower_left, upper_left)
        }
    }

    pub fn get_height_at_coordinates(&self, x: f64, z: f64) -> f64 {
        let [[min_x, max_x], [min_z, max_z]] = self.footprint;
        if x < min_x || x > max_x || z < min_z || z > max_z {
            return 0.0;
        }

        let (high_corner, low_corner) = self.get_high_and_low_corner();

        let incline_vector = high_corner - low_corner;
        let ramp_height_proportion = DVec2::new(x - min_x, z - min_z)
            .project_onto(incline_vector)
            .length()
            / incline_vector.length();

        self.min_height + (ramp_height_proportion * (self.max_height - self.min_height))
    }

    pub fn get_incline_vector(&self) -> DVec3 {
        let (high_corner, low_corner) = self.get_high_and_low_corner();

        let incline_vec = DVec3::new(low_corner.x, self.max_height, low_corner.y)
            - DVec3::new(high_corner.x, self.min_height, high_corner.y);
        incline_vec
    }
}
