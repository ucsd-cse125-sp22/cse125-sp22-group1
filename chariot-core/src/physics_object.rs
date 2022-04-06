/*
 * We'll limit ourselves to only modelling cars for the moment - while other
 * objects to which physics apply can be conceived of, cars are the only
 * ironclad one at the moment.
 */

use glam::DVec3;

#[derive(Copy, Clone)]
pub enum EngineStatus {
	Accelerating,
	Neutral,
	Braking
}

#[derive(Copy, Clone)]
pub enum RotationStatus {
	InSpinClockwise,
	InSpinCounterclockwise,
	NotInSpin
}

pub struct PhysicsProperties {
	pub position: DVec3,
	pub velocity: DVec3,
	pub mass: f64,

	// steering / controlled variables
	pub unit_steer_direction: DVec3, // should be a normalized vector
	pub angular_velocity: f64, // in radians per time unit
	pub engine_status: EngineStatus,
	pub rotation_status: RotationStatus,
}