use crate::checkpoints::*;

pub struct Map {
    pub major_zones: Vec<MajorCheckpoint>,
    pub checkpoints: Vec<MinorCheckpoint>,
    pub finish_line: FinishLine,
}
