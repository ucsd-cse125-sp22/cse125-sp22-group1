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

impl PlayerEntity {
    fn get_bounding_box_corners(&self) -> [[f64; 2]; 3] {
        let x_2 = self.x_size / 2.0;
        let y_2 = self.y_size / 2.0;
        let z_2 = self.z_size / 2.0;

        // Angles are measured from the (1, 0, 0) axis
        let heading = DVec2::new(
            self.entity_location.unit_steer_direction[0],
            self.entity_location.unit_steer_direction[2],
        );
        let zero_angle_vec = DVec2::new(1.0, 0.0);

        // Since the range of arccos is [0, pi], and bounding boxes are
        // reflectionally symmetrical over the X and Z axes, we can thus
        // constrain (for the purposes of bounding-box calculation) the rotation
        // angle to be in [0, pi/2].
        let mut theta: f64 = heading.dot(zero_angle_vec).acos();
        if theta > std::f64::consts::FRAC_PI_2 {
            theta -= std::f64::consts::FRAC_PI_2;
        }

        // These two vectors and -1 times these two vectors define the four
        // corners of the bounding box (source: trust me bro)
        let one_corner = flat_rotate_vector(&DVec2::new(x_2, z_2), theta);
        let other_corner =
            flat_rotate_vector(&DVec2::new(-x_2, -z_2), std::f64::consts::PI - theta);

        let xs = [
            one_corner.x,
            other_corner.x,
            (-1.0 * one_corner).x,
            (-1.0 * other_corner).x,
        ];
        let zs = [
            one_corner.y,
            other_corner.y,
            (-1.0 * one_corner).y,
            (-1.0 * other_corner).y,
        ];

        let min_x = self.entity_location.position.x + xs.into_iter().reduce(f64::min).unwrap();
        let max_x = self.entity_location.position.x + xs.into_iter().reduce(f64::max).unwrap();

        let min_z = self.entity_location.position.z + zs.into_iter().reduce(f64::min).unwrap();
        let max_z = self.entity_location.position.z + zs.into_iter().reduce(f64::max).unwrap();

        let min_y = self.entity_location.position.y - y_2;
        let max_y = self.entity_location.position.y + y_2;

        return [[min_x, max_x], [min_y, max_y], [min_z, max_z]];
    }
}
fn check_bounding_box_collisions(p1: &PlayerEntity, p2: &PlayerEntity) -> bool {
    let bounding_box_1 = p1.get_bounding_box_corners();
    let bounding_box_2 = p2.get_bounding_box_corners();

    let mut collision_dimensions = [false, false, false];

    for dimension in 0..=2 {
        let [min_1, max_1] = bounding_box_1[dimension];
        let [min_2, max_2] = bounding_box_2[dimension];

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

fn are_players_colliding(p1: &PlayerEntity, p2: &PlayerEntity) -> bool {
    return check_bounding_box_collisions(&p1, &p2) || check_bounding_box_collisions(&p2, &p1);
}

impl PlayerEntity {
    pub fn collide_players(&self, other: &PlayerEntity) -> Option<PlayerEntity> {
        if !are_players_colliding(&self, other) {
            return None;
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

        let new_v1 = v1 - term1 * term2 * term3;

        return Some(PlayerEntity {
            velocity: new_v1,
            angular_velocity: self.angular_velocity,
            mass: self.mass,
            x_size: self.x_size,
            y_size: self.y_size,
            z_size: self.z_size,
            player_inputs: self.player_inputs,
            entity_location: self.entity_location,
        });
    }
}

mod tests {
    use chariot_core::{
        entity_location::EntityLocation,
        player_inputs::{EngineStatus, PlayerInputs, RotationStatus},
    };
    use glam::DVec3;

    use crate::physics::{collisions::are_players_colliding, PlayerEntity};

    fn get_origin_cube() -> PlayerEntity {
        return PlayerEntity {
            player_inputs: PlayerInputs {
                engine_status: EngineStatus::Neutral,
                rotation_status: RotationStatus::NotInSpin,
            },

            entity_location: EntityLocation {
                position: DVec3::new(0.0, 0.0, 0.0),
                unit_steer_direction: DVec3::new(1.0, 0.0, 0.0),
            },

            velocity: DVec3::new(0.0, 0.0, 0.0),
            angular_velocity: 0.0,
            mass: 10.0,

            x_size: 10.0,
            y_size: 10.0,
            z_size: 10.0,
        };
    }

    #[test]
    fn test_collision_with_self() {
        let origin_cube = get_origin_cube();
        assert!(are_players_colliding(&origin_cube, &origin_cube));
    }

    #[test]
    fn test_engulfed_collision() {
        let big_origin_cube = get_origin_cube();
        let mut smol_origin_cube = get_origin_cube();
        smol_origin_cube.x_size = 1.0;
        smol_origin_cube.y_size = 1.0;
        smol_origin_cube.z_size = 1.0;
        assert!(are_players_colliding(&big_origin_cube, &smol_origin_cube))
    }

    #[test]
    fn test_collision_on_corner() {
        let origin_cube = get_origin_cube();
        let mut not_origin_cube = get_origin_cube();
        not_origin_cube.entity_location.position = DVec3::new(10.0, 10.0, 10.0);
        assert!(are_players_colliding(&origin_cube, &not_origin_cube))
    }

    #[test]
    fn test_noncollision_on_corner() {
        let origin_cube = get_origin_cube();
        let mut not_origin_cube = get_origin_cube();
        not_origin_cube.entity_location.position = DVec3::new(10.1, 10.1, 10.1);
        assert!(!are_players_colliding(&origin_cube, &not_origin_cube))
    }

    // we have different logic for the y-direction, might as well test that
    #[test]
    fn test_noncollision_when_above_or_below() {
        let origin_cube = get_origin_cube();
        let mut high_cube = get_origin_cube();
        let mut low_cube = get_origin_cube();
        high_cube.entity_location.position = DVec3::new(0.0, 20.0, 0.0);
        low_cube.entity_location.position = DVec3::new(0.0, -20.0, 0.0);
        assert!(!are_players_colliding(&origin_cube, &high_cube));
        assert!(!are_players_colliding(&origin_cube, &low_cube));
        assert!(!are_players_colliding(&high_cube, &low_cube)); // just for good measure
    }

    #[test]
    fn test_collision_when_above_or_below() {
        let origin_cube = get_origin_cube();
        let mut high_cube = get_origin_cube();
        let mut low_cube = get_origin_cube();
        high_cube.entity_location.position = DVec3::new(0.0, 8.0, 0.0);
        low_cube.entity_location.position = DVec3::new(0.0, -8.0, 0.0);
        assert!(are_players_colliding(&origin_cube, &high_cube));
        assert!(are_players_colliding(&origin_cube, &low_cube));
        assert!(!are_players_colliding(&high_cube, &low_cube)); // just for good measure
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
        assert!(are_players_colliding(&owo_cube, &uwu_cube));
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
        assert!(are_players_colliding(&owo_cube, &uwu_cube));
    }
}
