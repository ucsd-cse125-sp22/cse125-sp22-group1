use serde::{Deserialize, Serialize};

pub type LapNumber = u8;
pub type CheckpointID = u8;
pub type ZoneID = u8;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct LapInformation {
    pub lap: LapNumber,
    pub last_checkpoint: MinorCheckpointID,
    pub zone: MajorCheckpointID,
}

impl LapInformation {
    pub fn new() -> Self {
        LapInformation {
            lap: 0,
            last_checkpoint: 0,
            zone: 0,
        }
    }
}
