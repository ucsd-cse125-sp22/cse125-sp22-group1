use crate::physics::bounding_box::BoundingBox;
use crate::physics::{player_entity::PlayerEntity, trigger_entity::TriggerEntity};
use chariot_core::lap_info::*;
use glam::DVec3;

#[derive(Clone, Copy)]
pub struct Checkpoint {
    pub id: CheckpointID,
    pub pos: DVec3,
    pub size: DVec3,
    pub bounding_box: BoundingBox,
}

impl TriggerEntity for Checkpoint {
    fn get_bounding_box(&self) -> BoundingBox {
        self.bounding_box
    }
    fn trigger(&self, ply: &mut PlayerEntity) {
        ply.lap_info.last_checkpoint = self.id;
    }
}

#[derive(Clone, Copy)]
pub struct Zone {
    pub id: ZoneID,
    pos: DVec3,
    size: DVec3,
    pub bounding_box: BoundingBox,
}

impl Zone {
    pub fn new(id: ZoneID, pos: DVec3, size: DVec3) -> Self {
        Self {
            id: id,
            pos: pos,
            size: size,
            bounding_box: BoundingBox::new(
                pos.x,
                pos.x + size.x,
                pos.y,
                pos.y + size.y,
                pos.z,
                pos.z + size.z,
            ),
        }
    }
}

impl TriggerEntity for Zone {
    fn get_bounding_box(&self) -> BoundingBox {
        self.bounding_box
    }

    fn trigger(&self, ply: &mut PlayerEntity) {
        // Only advance zone if the player is in the zone before us
        if (ply.lap_info.zone + 1) == self.id {
            ply.lap_info.zone = self.id;
        }
    }
}

#[derive(Clone, Copy)]
pub struct FinishLine {
    last_zone: ZoneID,
    pos: DVec3,
    size: DVec3,
    pub bounding_box: BoundingBox,
}

impl FinishLine {
    pub fn new(pos: DVec3, size: DVec3, last_zone: ZoneID) -> Self {
        Self {
            last_zone,
            pos,
            size,
            bounding_box: BoundingBox::new(
                pos.x,
                pos.x + size.x,
                pos.y,
                pos.y + size.y,
                pos.z,
                pos.z + size.z,
            ),
        }
    }
}

impl TriggerEntity for FinishLine {
    fn get_bounding_box(&self) -> BoundingBox {
        self.bounding_box
    }

    fn trigger(&self, ply: &mut PlayerEntity) {
        // Player is only allowed to advance if they are on the track's last zone
        if ply.lap_info.zone == self.last_zone {
            ply.lap_info.zone = 0;
        }
    }
}
