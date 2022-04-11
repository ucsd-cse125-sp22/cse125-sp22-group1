use glam::{DVec2, DVec3};

use super::player_entity::PlayerEntity;

// Given a 3D vector, rotate it in the XZ plane by theta radians counterclockwise.
// Refer to https://en.wikipedia.org/wiki/Rotation_matrix for the formula used here
fn flat_rotate_vector(v: &DVec3, theta: f64) -> DVec3 {
    let x = v[0];
    let y = v[1];
    let z = v[2];

    let sin_theta = theta.sin();
    let cos_theta = theta.cos();

    return DVec3::new(
        x * cos_theta - z * sin_theta,
        y,
        x * sin_theta + z * cos_theta,
    );
}

impl PlayerEntity {
    // Given the hitbox of an entity, return the coordinates of its corners when
    // rotated counterclockwise in the XZ plane by a given number of radians
    fn get_corners_of_hitbox(&self) -> [DVec3; 8] {
        let x_2 = self.x_size / 2.0;
        let y_2 = self.y_size / 2.0;
        let z_2 = self.z_size / 2.0;

        // Angles are measured from the (1, 0, 0) axis
        let heading = DVec2::new(
            self.entity_location.unit_steer_direction[0],
            self.entity_location.unit_steer_direction[2],
        );
        let zero_angle_vec = DVec2::new(1.0, 0.0);

        // Since the range of arccos is [0, pi], we add the extra pi if turned around more than that
        let theta;
        if heading[1] >= 0.0 {
            theta = heading.dot(zero_angle_vec).acos();
        } else {
            theta = heading.dot(zero_angle_vec).acos() + std::f64::consts::PI;
        }

        let center = DVec3::new(
            self.entity_location.position[0],
            self.entity_location.position[1],
            self.entity_location.position[2],
        );

        // If the center were at (0, 0, 0), then each corner of the hitbox would
        // have coordinates of the form (+-x_2, +-y_2, +-z_2) -- each corner has
        // a unique combination of positive and negative signs for the
        // coordinates. We rotate all those vectors by the given angle, and then
        // translate by the center of the object to get the corners of the real
        // hitbox.

        return [
            center + flat_rotate_vector(&DVec3::new(x_2, y_2, z_2), theta),
            center + flat_rotate_vector(&DVec3::new(x_2, y_2, -z_2), theta),
            center + flat_rotate_vector(&DVec3::new(x_2, -y_2, z_2), theta),
            center + flat_rotate_vector(&DVec3::new(x_2, -y_2, -z_2), theta),
            center + flat_rotate_vector(&DVec3::new(x_2, y_2, z_2), theta),
            center + flat_rotate_vector(&DVec3::new(x_2, y_2, -z_2), theta),
            center + flat_rotate_vector(&DVec3::new(x_2, -y_2, z_2), theta),
            center + flat_rotate_vector(&DVec3::new(x_2, -y_2, -z_2), theta),
        ];
    }
}

// Check whether any portion of p1 is within p2
fn check_collision(p1: &PlayerEntity, p2: &PlayerEntity) -> bool {
    let p1_corners = p1.get_corners_of_hitbox();
    let p2_corners = p2.get_corners_of_hitbox();

    // Two steps: check if the y-value is fine (easy cause we're always
    // spinning flat), and if it is then check whether (x, z) is within the 2d
    // rectangle

    for p1_corner in p1_corners {
        let high_y = p2_corners[0][1];
        let low_y = p2_corners[2][1];

        // If not in the y-range, this corner can't be in the bounding box; move on to the next corner
        if p1_corner[1] > high_y || p1_corner[1] < low_y {
            continue;
        }

        // Otherwise, check using the 2d rectangle defined by x's and z's
        // I do not pretend to understand this formula, but it's taken from
        // https://math.stackexchange.com/a/190373
        let M_x = p1_corner[0];
        let M_z = p1_corner[2];

        let A_x = p2_corners[0][0];
        let A_z = p2_corners[0][2];
        let B_x = p2_corners[1][0];
        let B_z = p2_corners[1][2];
        let D_x = p2_corners[4][0];
        let D_z = p2_corners[4][2];

        let AM = DVec2::new(M_x - A_x, M_z - A_z);
        let AB = DVec2::new(B_x - A_x, B_z - A_z);
        let AD = DVec2::new(D_x - A_x, D_z - A_z);

        if 0.0 <= AM.dot(AB)
            && AM.dot(AB) <= AB.dot(AB)
            && 0.0 <= AM.dot(AD)
            && AM.dot(AD) <= AD.dot(AD)
        {
            return true;
        }
    }

    return false;
}

fn are_players_colliding(p1: &PlayerEntity, p2: &PlayerEntity) -> bool {
    return check_collision(&p1, &p2) || check_collision(&p2, &p1);
}

pub fn collide_players(
    p1: PlayerEntity,
    p2: PlayerEntity,
    time_step: f64,
) -> (PlayerEntity, PlayerEntity) {
    if !are_players_colliding(&p1, &p2) {
        return (p1, p2);
    }

    // Given two force vectors a, b corresponding to objects A, B colliding: the
    // force on A is equal to a - proj_b a, since proj_b a corresponds to the
    // component of force acting in the same direction. Similarly, the force on
    // B is b - proj_a b.

    let p1_momentum = p1.velocity * p1.mass;
    let p2_momentum = p2.velocity * p2.mass;
    let delta_p1_momentum = p1_momentum - p1_momentum.project_onto(p2_momentum);
    let delta_p2_momentum = p2_momentum - p2_momentum.project_onto(p1_momentum);

    let new_pe = PlayerEntity {
        velocity: p1.velocity,
        angular_velocity: p1.angular_velocity,
        mass: p1.mass,
        x_size: p1.x_size,
        y_size: p1.y_size,
        z_size: p1.z_size,
        player_inputs: p1.player_inputs,
        entity_location: p1.entity_location,
    };

    return (new_pe, p2);
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
