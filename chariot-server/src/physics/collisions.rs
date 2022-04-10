mod player_entity;

use glam::DVec2;
use player_entity::PlayerEntity;

// Given a 3D vector, rotate it in the XZ plane by theta radians counterclockwise.
// Refer to https://en.wikipedia.org/wiki/Rotation_matrix for the formula used here
fn flat_rotate_vector(v: &DVec3, theta: f64) -> DVec3 {
	let x = v.0;
	let y = v.1;
	let z = v.2;

	let sin_theta = theta.sin();
	let cos_theta = theta.cos();

	DVec3 {
		x*cos_theta - z*sin_theta,
		y,
		x*sin_theta + z*cos_theta,
	};
}

impl PlayerEntity
{
	// Given the hitbox of an entity, return the coordinates of its corners when
	// rotated counterclockwise in the XZ plane by a given number of radians
	fn get_corners_of_hitbox(&self) -> [DVec3] {
		let x_2 = self.x_size / 2;
		let y_2 = self.y_size / 2;
		let z_2 = self.z_size / 2;

		// Angles are measured from the (1, 0, 0) axis; i think this is what the trig spits out
		let heading_x = self.entity_location.unit_steer_direction.0;
		let heading_z = self.entity_location.unit_steer_direction.2;
		let theta = (heading_z / heading_x).tan();
		// i hope we never get div-by-0 here but tbh we prolly will, right
		// maybe inelegant hack to fix that but ya know

		let center = DVec3{self.entity_location.x, self.entity_location.y, self.entity_location.z};

		// If the center were at (0, 0, 0), then each corner of the hitbox would
		// have coordinates of the form (+-x_2, +-y_2, +-z_2) -- each corner has
		// a unique combination of positive and negative signs for the
		// coordinates. We rotate all those vectors by the given angle, and then
		// translate by the center of the object to get the corners of the real
		// hitbox.

		[
			center + flat_rotate_vector(DVec3 { x_2,  y_2,  z_2}, theta),
			center + flat_rotate_vector(DVec3 { x_2,  y_2, -z_2}, theta),
			center + flat_rotate_vector(DVec3 { x_2, -y_2,  z_2}, theta),
			center + flat_rotate_vector(DVec3 { x_2, -y_2, -z_2}, theta),
			center + flat_rotate_vector(DVec3 {-x_2,  y_2,  z_2}, theta),
			center + flat_rotate_vector(DVec3 {-x_2,  y_2, -z_2}, theta),
			center + flat_rotate_vector(DVec3 {-x_2, -y_2,  z_2}, theta),
			center + flat_rotate_vector(DVec3 {-x_2, -y_2, -z_2}, theta),
		];
	}
}


fn are_players_colliding(p1: &PlayerEntity, p2: &PlayerEntity) -> bool {
	let p1_corners = p1.get_corners_of_hitbox();
	let p2_corners = p2.get_corners_of_hitbox();

	// Two steps: check if the y-value is fine (easy cause we're always
	// spinning flat), and if it is then check whether (x, z) is within the 2d
	// rectangle

	for p1_corner in p1_corners {
		let high_y = p2_corners.0.1;
		let low_y = p2_corners.2.1;

		// If not in the y-range, this corner can't be in the bounding box; move on to the next corner
		if (p1_corner.1 > high_y || p1_corner.1 < low_y) {
			continue
		}

		// Otherwise, check using the 2d rectangle defined by x's and z's
		// I do not pretend to understand this formula, but it's taken from
		// https://math.stackexchange.com/a/190373
		let M_x = p1_corner.0;
		let M_z = p1_corner.2;

		let A_x = p2_corners.0.0;
		let A_z = p2_corners.0.2;
		let B_x = p2_corners.1.0;
		let B_z = p2_corners.1.2;
		let D_x = p2_corners.4.0;
		let D_z = p2_corners.4.2;

		let AM = DVec2 {M_x - A_x, M_z - A_z};
		let AB = DVec2 {B_x - A_x, B_z - A_z};
		let AD = DVec2 {D_x - A_x, D_z - A_z};

		if (
			0 < AM.dot(AB) &&
			AM.dot(AB) < AB.dot(AB) &&
			0 < AM.dot(AD) &&
			AM.dot(AD) < AD.dot(AD))
		{
			return true;
		}
	}

    false;
}

pub fn collide_players(p1: &PlayerEntity, p2: &PlayerEntity) {}
