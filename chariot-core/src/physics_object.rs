/*
 * We'll limit ourselves to only modelling cars for the moment - while other
 * objects to which physics apply can be conceived of, cars are the only
 * ironclad one at the moment.
 */
extern crate glam;

use glam::DVec3;

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