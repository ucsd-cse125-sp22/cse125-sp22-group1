use glam::{DVec2, DVec3};

use super::player_entity::PlayerEntity;

// Given a 2D vector, rotate it by theta radians counterclockwise.
// Refer to https://en.wikipedia.org/wiki/Rotation_matrix for the formula used here
fn flat_rotate_vector(v: &DVec2, theta: f64) -> DVec2 {
    let x = v[0];
    let z = v[1];

    let sin_theta = theta.sin();
    let cos_theta = theta.cos();

    return DVec2::new(x * cos_theta - z * sin_theta, x * sin_theta + z * cos_theta);
}

// Assume we have a basis in a 2D plane. Then we're given two vectors in this
// plane, both starting at the origin; one defines which direction an object is
// facing in the plane, and the other defines where the corner of the object
// within this plane would be facing if the object was unrotated (pointing
// towards [1, 0]). Use the rotation of the object to determine by how much the
// bounding box would need to grow from the unrotated dimensions of the object
// in order to fully enclose the object; do this in two dimensions and return
// how much needs to be added to the bounding box in each dimension.
fn get_rotated_extremum_distance_in_plane(
    direction_in_plane: &DVec2,
    corner_to_rotate: &DVec2,
) -> (f64, f64) {
    let zero_angle_vec = DVec2::new(1.0, 0.0);

    // Since the range of arccos is [0, pi], and bounding boxes are
    // reflectionally symmetrical over the X and Z axes, we can thus
    // constrain (for the purposes of bounding-box calculation) the rotation
    // angle to be in [0, pi/2].
    let mut theta: f64 = direction_in_plane.dot(zero_angle_vec).acos();
    if theta > std::f64::consts::FRAC_PI_2 {
        theta -= std::f64::consts::FRAC_PI_2;
    }

    // These two vectors and -1 times these two vectors define the four
    // corners of the bounding box (source: trust me bro)
    let one_corner = flat_rotate_vector(corner_to_rotate, theta);
    let other_corner =
        flat_rotate_vector(&(*corner_to_rotate * -1.0), std::f64::consts::PI - theta);

    let xs = [
        one_corner.x,
        other_corner.x,
        (-1.0 * one_corner).x,
        (-1.0 * other_corner).x,
    ];
    let ys = [
        one_corner.y,
        other_corner.y,
        (-1.0 * one_corner).y,
        (-1.0 * other_corner).y,
    ];

    let x_dist = xs.into_iter().reduce(f64::max).unwrap();
    let y_dist = ys.into_iter().reduce(f64::max).unwrap();

    return (x_dist, y_dist);
}

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

        // Angles in the XZ-plane are measured from the (1, 0, 0) axis
        let xz_heading = DVec2::new(
            self.entity_location.unit_steer_direction[0],
            self.entity_location.unit_steer_direction[2],
        );
        let xz_corner = DVec2::new(x_2, z_2);

        // Since the extremum distance in the y-direction is going to be the
        // same no matter how much we spin the body about the upward direction,
        // we can use any plane which contains the upward direction. The easy
        // choice is to pick the plane that also goes through the origin and
        // (x_2, y_2, z_2). For ease of basis (because the zero-angle vector is
        // (1, 0)), we put y first; call the other component of the basis (the
        // one that goes through (x_2, z_2)) w.

        // Sign doesn't matter here, we're rotationally symmetric
        let upward_direction_w = (self.entity_location.unit_upward_direction[0].powi(2)
            + self.entity_location.unit_upward_direction[2].powi(2))
        .sqrt();

        let yw_heading = DVec2::new(
            self.entity_location.unit_upward_direction[1],
            upward_direction_w,
        );
        let w_2 = (x_2.powi(2) + z_2.powi(2)).sqrt();

        let yw_corner = DVec2::new(y_2, w_2);

        // We can find the dimensions of the three-dimensional bounding box by
        // first rotating the object in the XZ-plane to get the X and Z extrema,
        // and then separately rotating in a Y-based plane to get the Y extrema.
        // By doing these separately, we avoid three-dimension rotation (which
        // sucks)
        let (x_dist, z_dist) = get_rotated_extremum_distance_in_plane(&xz_heading, &xz_corner);
        let (y_dist, _) = get_rotated_extremum_distance_in_plane(&yw_heading, &yw_corner);

        // This will always be nonnegative (since we're centered around the origin)
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
