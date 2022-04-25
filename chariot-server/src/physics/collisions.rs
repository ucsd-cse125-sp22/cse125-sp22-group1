use glam::{DVec3, Mat3};

use super::player_entity::PlayerEntity;

impl PlayerEntity {
    fn check_bounding_box_collisions(&self, other: &PlayerEntity) -> bool {
        let mut collision_dimensions = [false, false, false];

        for dimension in 0..=2 {
            let [min_1, max_1] = self.bounding_box[dimension];
            let [min_2, max_2] = other.bounding_box[dimension];

            if {
                (min_2 <= min_1 && min_1 <= max_2) // min_1 is inside 2
					|| (min_2 <= max_1 && max_1 <= max_2) // max_1 is inside 2
					|| (min_1 <= min_2 && min_2 <= max_1) // min_2 is inside 1
					|| (min_1 <= max_2 && max_2 <= max_1) // max_2 is inside 1
            } {
                collision_dimensions[dimension] = true;
            }
        }

        return collision_dimensions.iter().all(|&x| x);
    }

    pub fn set_bounding_box_dimensions(&mut self) {
        let x_2 = self.size.x / 2.0;
        let y_2 = self.size.y / 2.0;
        let z_2 = self.size.z / 2.0;

        // unit_steer_direction defines yaw, unit_upward_direction can be
        // decomposed into pitch and roll; with these, we can get Euler angles
        // for the 3d rotation. and then, to compute the bounding box we can
        // literally just rotate the corners of the object and find the extrema!

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

        let yaw_rotation_matrix = Mat3::from_rotation_y(yaw as f32);
        let pitch_rotation_matrix = Mat3::from_rotation_z(pitch as f32);
        let roll_rotation_matrix = Mat3::from_rotation_x(roll as f32);

        // because of symmetry, we only need to rotate four corners all on the same face; doesn't matter which face
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

        let min_x = self.entity_location.position.x - x_dist;
        let max_x = self.entity_location.position.x + x_dist;

        let min_y = self.entity_location.position.y - y_dist;
        let max_y = self.entity_location.position.y + y_dist;

        let min_z = self.entity_location.position.z - z_dist;
        let max_z = self.entity_location.position.z + z_dist;

        self.bounding_box = [[min_x, max_x], [min_y, max_y], [min_z, max_z]];
    }

    // Returns the velocity change to self from colliding with other
    pub fn delta_v_from_collision_with_player(&self, other: &PlayerEntity) -> DVec3 {
        if !self.check_bounding_box_collisions(other) {
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

        let term1 = (-2.0 * m2) / (m1 + m2);
        let term2 = (v1 - v2).dot(x1 - x2) / (x1 - x2).length_squared();
        let term3 = x1 - x2;

        return term1 * term2 * term3;
    }
}

#[cfg(test)]
mod tests {
    use chariot_core::{
        entity_location::EntityLocation,
        player_inputs::{EngineStatus, PlayerInputs, RotationStatus},
    };
    use glam::DVec3;

    use crate::physics::PlayerEntity;

    fn get_origin_cube() -> PlayerEntity {
        return PlayerEntity {
            player_inputs: PlayerInputs {
                engine_status: EngineStatus::Neutral,
                rotation_status: RotationStatus::NotInSpin,
            },

            entity_location: EntityLocation {
                position: DVec3::new(0.0, 0.0, 0.0),
                unit_steer_direction: DVec3::new(1.0, 0.0, 0.0),
                unit_upward_direction: DVec3::new(0.0, 1.0, 0.0),
            },

            velocity: DVec3::new(0.0, 0.0, 0.0),
            angular_velocity: 0.0,
            mass: 10.0,

            size: DVec3::new(10.0, 10.0, 10.0),
            bounding_box: [[-5.0, 5.0], [-5.0, 5.0], [-5.0, 5.0]],
            physics_changes: Vec::new(),
        };
    }

    #[test]
    fn test_collision_with_self() {
        let origin_cube = get_origin_cube();
        assert!(origin_cube.check_bounding_box_collisions(&origin_cube));
    }

    #[test]
    fn test_engulfed_collision() {
        let big_origin_cube = get_origin_cube();
        let mut smol_origin_cube = get_origin_cube();
        smol_origin_cube.size = DVec3::new(1.0, 1.0, 1.0);
        smol_origin_cube.set_bounding_box_dimensions();
        assert!(big_origin_cube.check_bounding_box_collisions(&smol_origin_cube));
    }

    #[test]
    fn test_collision_on_corner() {
        let origin_cube = get_origin_cube();
        let mut not_origin_cube = get_origin_cube();
        not_origin_cube.entity_location.position = DVec3::new(10.0, 10.0, 10.0);
        not_origin_cube.set_bounding_box_dimensions();
        assert!(origin_cube.check_bounding_box_collisions(&not_origin_cube));
    }

    #[test]
    fn test_noncollision_on_corner() {
        let origin_cube = get_origin_cube();
        let mut not_origin_cube = get_origin_cube();
        not_origin_cube.entity_location.position = DVec3::new(10.1, 10.1, 10.1);
        not_origin_cube.set_bounding_box_dimensions();
        assert!(!origin_cube.check_bounding_box_collisions(&not_origin_cube));
    }

    // we have different logic for the y-direction, might as well test that
    #[test]
    fn test_noncollision_when_above_or_below() {
        let origin_cube = get_origin_cube();
        let mut high_cube = get_origin_cube();
        let mut low_cube = get_origin_cube();
        high_cube.entity_location.position = DVec3::new(0.0, 20.0, 0.0);
        low_cube.entity_location.position = DVec3::new(0.0, -20.0, 0.0);
        high_cube.set_bounding_box_dimensions();
        low_cube.set_bounding_box_dimensions();
        assert!(!origin_cube.check_bounding_box_collisions(&high_cube));
        assert!(!origin_cube.check_bounding_box_collisions(&low_cube));
        assert!(!high_cube.check_bounding_box_collisions(&low_cube)); // just for good measure
    }

    #[test]
    fn test_collision_when_above_or_below() {
        let origin_cube = get_origin_cube();
        let mut high_cube = get_origin_cube();
        let mut low_cube = get_origin_cube();
        high_cube.entity_location.position = DVec3::new(0.0, 8.0, 0.0);
        low_cube.entity_location.position = DVec3::new(0.0, -8.0, 0.0);
        high_cube.set_bounding_box_dimensions();
        low_cube.set_bounding_box_dimensions();
        assert!(origin_cube.check_bounding_box_collisions(&high_cube));
        assert!(origin_cube.check_bounding_box_collisions(&low_cube));
        assert!(!high_cube.check_bounding_box_collisions(&low_cube)); // just for good measure
    }

    #[test]
    fn test_collision_on_rotated_edges() {
        // uwu w-wat if i was a cube with edge wength 10 (⁄ ⁄•⁄ω⁄•⁄ ⁄) centewed
        // at the o-owigin and wotated 45 degwees (>ω^)
        // a-and u (☆ω☆) wewe a cube with edge wength 10 a-awso wotated 45
        // degwees (o･ω･o) but c-centewed 10sqwt(2) units away in the
        // x-diwection Owo a-and we 😳 👉 👈 t-touched edges 🥺
        let mut owo_cube = get_origin_cube();
        let mut uwu_cube = get_origin_cube();

        owo_cube.entity_location.unit_steer_direction =
            DVec3::new(2.0_f64.sqrt() / 2.0, 0.0, 2.0_f64.sqrt() / 2.0);
        uwu_cube.entity_location.unit_steer_direction =
            DVec3::new(2.0_f64.sqrt() / 2.0, 0.0, 2.0_f64.sqrt() / 2.0);

        uwu_cube.entity_location.position = DVec3::new(10.0 * 2.0_f64.sqrt() - 0.1, 0.0, 0.0);
        uwu_cube.set_bounding_box_dimensions();
        owo_cube.set_bounding_box_dimensions();
        assert!(uwu_cube.check_bounding_box_collisions(&owo_cube));
    }

    #[test]
    fn test_collision_on_30_deg_rotated_edges() {
        // uwu owo yadda yadda
        let mut owo_cube = get_origin_cube();
        let mut uwu_cube = get_origin_cube();

        // upper right corner is located at x = 5sqrt(6) / 2, z = 5sqrt(2) / 2
        owo_cube.entity_location.unit_steer_direction =
            DVec3::new(1.0 / 2.0, 0.0, 3.0_f64.sqrt() / 2.0);
        uwu_cube.entity_location.unit_steer_direction =
            DVec3::new(-1.0 / 2.0, 0.0, 3.0_f64.sqrt() / 2.0);

        uwu_cube.entity_location.position = DVec3::new(5.0 * 6.0_f64.sqrt(), 0.0, 0.0);
        uwu_cube.set_bounding_box_dimensions();
        owo_cube.set_bounding_box_dimensions();
        assert!(uwu_cube.check_bounding_box_collisions(&owo_cube));
    }

    #[test]
    fn test_3d_bounding_box() {
        let mut cube = get_origin_cube();

        cube.size = DVec3::new(1.0, 10000.0, 1.0);
        cube.entity_location.unit_upward_direction =
            DVec3::new(2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0, 0.0);
        cube.set_bounding_box_dimensions();

        let y_max = cube.bounding_box[1][1];
        let y_min = cube.bounding_box[1][0];

        let actual_top = (10_000.0 / 2.0) / (2.0_f64.sqrt());
        let actual_bottom = (-10_000.0 / 2.0) / (2.0_f64.sqrt());
        assert!(actual_top * 0.999 < y_max && y_max < actual_top * 1.001);
        assert!(actual_bottom * 0.999 > y_min && y_min > actual_bottom * 1.001);
    }
}
