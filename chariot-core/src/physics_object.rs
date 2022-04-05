/*
 * We'll limit ourselves to only modelling cars for the moment - while other
 * objects to which physics apply can be conceived of, cars are the only
 * ironclad one at the moment.
 */
extern crate glam;

use glam::DVec3;

/*
#[derive(Copy, Clone, Debug)]
pub struct Vec3D {
	pub x: f64,
	pub y: f64,
	pub z: f64
}

impl std::ops::Add for Vec3D {
	type Output = Self;

	fn add(self, rhs: Vec3D) -> Vec3D {
		return Vec3D {x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z};
	}
}

impl std::ops::Mul<f64> for Vec3D {
	type Output = Self;

	fn mul(self, rhs: f64) -> Vec3D {
		return Vec3D{x: self.x * rhs, y: self.y * rhs, z: self.z * rhs};
	}
}

impl std::ops::Div<f64> for Vec3D {
	type Output = Self;

	fn div(self, rhs: f64) -> Vec3D {
		return Vec3D{x: self.x / rhs, y: self.y / rhs, z: self.z / rhs};
	}
}

fn close_enough(a: f64, b: f64) -> bool {
	return b <= 1.00001 * a && b >= 0.99999 * a;
}

impl PartialEq for Vec3D {
    fn eq(&self, other: &Self) -> bool {
		return close_enough(self.x, other.x) &&
		close_enough(self.y, other.y) &&
		close_enough(self.z, other.z)
    }
}

pub fn magnitude_Vec3D(vec: &Vec3D) -> f64 {
	return (vec.x*vec.x + vec.y*vec.y + vec.z*vec.z).sqrt();
}

pub fn normalize_Vec3D(vec: &Vec3D) -> Vec3D {
	return *vec / magnitude_Vec3D(vec);
}
*/

#[derive(Copy, Clone)]
pub enum EngineStatus {
	ACCELERATING,
	NEUTRAL,
	BRAKING
}

pub struct PhysicsProperties {
	pub position: DVec3,
	pub velocity: DVec3,

	pub linear_momentum: DVec3, // redundant with velocity; both are used for convenience's sake
	pub angular_momentum: DVec3,

	pub mass: f64,

	// steering / controlled variables

	pub unit_steer_direction: DVec3, // should be a normalized vector
	pub engine_status: EngineStatus,

}