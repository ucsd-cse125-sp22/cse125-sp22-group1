use serde::{Deserialize, Serialize};

type Modifier = f32;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    Engine(EngineStatus),
    Rotation(RotationStatus),
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum EngineStatus {
    Accelerating(Modifier),
    Neutral,
    Braking,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum RotationStatus {
    InSpinClockwise(Modifier),
    InSpinCounterclockwise(Modifier),
    NotInSpin,
}

// PlayerInputs gets sent from the client to the server to inform the simulation
// about what a player is doing
#[derive(Copy, Clone)]
pub struct PlayerInputs {
    pub engine_status: EngineStatus,
    pub rotation_status: RotationStatus,
}
