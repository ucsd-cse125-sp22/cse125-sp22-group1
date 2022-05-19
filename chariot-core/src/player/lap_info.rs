use serde::{Deserialize, Serialize};

pub type LapNumber = u8;
pub type CheckpointID = u64;
pub type ZoneID = u64;
pub type Placement = u8;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct LapInformation {
    pub lap: LapNumber,
    pub zone: ZoneID,
    pub last_checkpoint: CheckpointID,
    pub placement: Placement,
}

impl LapInformation {
    pub fn new() -> Self {
        LapInformation {
            lap: 1,
            zone: 0,
            last_checkpoint: 0,
            placement: 0,
        }
    }
}
