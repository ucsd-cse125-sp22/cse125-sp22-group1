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
