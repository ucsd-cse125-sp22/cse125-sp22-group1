/*
 * We'll limit ourselves to only modelling cars for the moment - while other
 * objects to which physics apply can be conceived of, cars are the only
 * ironclad one at the moment.
 */

#[derive(Copy, Clone)]
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

pub struct PhysicsProperties {
	pub position: Vec3D,
	pub linear_momentum: Vec3D,
	pub angular_momentum: Vec3D,

	pub mass: f64
}