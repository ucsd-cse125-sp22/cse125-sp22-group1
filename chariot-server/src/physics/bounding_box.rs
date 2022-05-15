use std::ops::Bound;

use glam::{DVec3, Mat3};

#[derive(Copy, Clone, Debug)]
pub struct BoundingBox {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

impl BoundingBox {
    pub fn new(
        min_x: f64,
        max_x: f64,
        min_y: f64,
        max_y: f64,
        min_z: f64,
        max_z: f64,
    ) -> BoundingBox {
        BoundingBox {
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z,
        }
    }

    pub fn from_vecs(min: DVec3, max: DVec3) -> BoundingBox {
        BoundingBox {
            min_x: min.x,
            max_x: max.x,
            min_y: min.y,
            max_y: max.y,
            min_z: min.z,
            max_z: max.z,
        }
    }

    pub fn extremes() -> BoundingBox {
        BoundingBox {
            min_x: f64::MAX,
            max_x: f64::MIN,
            min_y: f64::MAX,
            max_y: f64::MIN,
            min_z: f64::MAX,
            max_z: f64::MIN,
        }
    }

    pub fn accum(&self, new: BoundingBox) -> BoundingBox {
        BoundingBox {
            min_x: self.min_x.min(new.min_x),
            max_x: self.max_x.max(new.max_x),
            min_y: self.min_y.min(new.min_y),
            max_y: self.max_y.max(new.max_y),
            min_z: self.min_z.min(new.min_z),
            max_z: self.max_z.max(new.max_z),
        }
    }

    pub fn pos(&self) -> DVec3 {
        DVec3::new(
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
            (self.min_z + self.max_z) / 2.0,
        )
    }

    pub fn is_colliding(&self, other: &BoundingBox) -> bool {
        // https://developer.mozilla.org/en-US/docs/Games/Techniques/3D_collision_detection
        (self.min_x <= other.max_x && self.max_x >= other.min_x)
            && (self.min_y <= other.max_y && self.max_y >= other.min_y)
            && (self.min_z <= other.max_z && self.max_z >= other.min_z)
    }

    // update this bounding box based on the provided position, size, pitch, yaw, and roll
    pub fn set_dimensions(&mut self, pos: &DVec3, size: &DVec3, pitch: f64, yaw: f64, roll: f64) {
        // with pitch and yaw, we can get Euler angles
        // for the 3d rotation. and then, to compute the bounding box we can
        // literally just rotate the corners of the object and find the extrema!
        let yaw_rotation_matrix = glam::Mat3::from_rotation_y(yaw as f32);
        let pitch_rotation_matrix = Mat3::from_rotation_z(pitch as f32);
        let roll_rotation_matrix = Mat3::from_rotation_x(roll as f32);

        // because of symmetry, we only need to rotate four corners all on the same face; doesn't matter which face
        let x_2 = size.x / 2.0;
        let y_2 = size.y / 2.0;
        let z_2 = size.z / 2.0;
        let corners = [
            DVec3::new(x_2, y_2, z_2),
            DVec3::new(-x_2, y_2, z_2),
            DVec3::new(x_2, y_2, -z_2),
            DVec3::new(-x_2, y_2, -z_2),
        ];

        // order is important and we want extrinsic rotation. then the order we
        // want, as per wikipedia, is yaw, then pitch, then roll - read this
        // from inside out
        let corners_coordinates = corners.iter().map(|corner| {
            roll_rotation_matrix.mul_vec3(
                pitch_rotation_matrix.mul_vec3(yaw_rotation_matrix.mul_vec3(corner.as_vec3())),
            )
        });

        // symmetry! max in one direction is min in the other direction
        let (mut x_dist, mut y_dist, mut z_dist) = (0.0, 0.0, 0.0);
        for rotated_corner in corners_coordinates {
            if rotated_corner.x.abs() > x_dist as f32 {
                x_dist = f64::from(rotated_corner.x);
            }
            if rotated_corner.y.abs() > y_dist as f32 {
                y_dist = f64::from(rotated_corner.y);
            }
            if rotated_corner.z.abs() > z_dist as f32 {
                z_dist = f64::from(rotated_corner.z);
            }
        }

        self.min_x = pos.x - x_dist;
        self.max_x = pos.x + x_dist;
        self.min_y = pos.y - y_dist;
        self.max_y = pos.y + y_dist;
        self.min_z = pos.z - z_dist;
        self.max_z = pos.z + z_dist;
    }
}
