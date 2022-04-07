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

// PlayerInputs gets sent from the client to the server to inform the simulation
// about what a player is doing
pub struct PlayerInputs {
	pub engine_status: EngineStatus,
	pub rotation_status: RotationStatus,
}

// EntityLocation gets sent back from the server to the client to give it
// information on the results of the simulation == where to render players
pub struct EntityLocation {
	pub position: DVec3,
	pub unit_steer_direction: DVec3, // should be a normalized vector
}