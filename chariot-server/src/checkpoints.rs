use crate::physics::bounding_box::BoundingBox;
use crate::physics::{player_entity::PlayerEntity, trigger_entity::TriggerEntity};
use chariot_core::lap_info::*;
use glam::DVec3;

#[derive(Clone, Copy)]
pub struct Checkpoint {
    pub id: CheckpointID,
    pub bounding_box: BoundingBox,
}

impl Checkpoint {
    pub fn new(id: CheckpointID, min: DVec3, max: DVec3) -> Self {
        Self {
            id,
            bounding_box: BoundingBox::from_vecs(min, max),
        }
    }
}

impl TriggerEntity for Checkpoint {
    fn pos(&self) -> DVec3 {
        self.bounding_box.pos()
    }

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
    pub bounding_box: BoundingBox,
}

impl Zone {
    pub fn new(id: ZoneID, min: DVec3, max: DVec3) -> Self {
        Self {
            id,
            bounding_box: BoundingBox::from_vecs(min, max),
        }
    }
}

impl TriggerEntity for Zone {
    fn pos(&self) -> DVec3 {
        self.bounding_box.pos()
    }

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
    pub bounding_box: BoundingBox,
}

impl FinishLine {
    pub fn new(min: DVec3, max: DVec3, last_zone: ZoneID) -> Self {
        Self {
            last_zone,
            bounding_box: BoundingBox::from_vecs(min, max),
        }
    }
}

impl TriggerEntity for FinishLine {
    fn pos(&self) -> DVec3 {
        self.bounding_box.pos()
    }

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
