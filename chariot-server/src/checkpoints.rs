use crate::physics::bounding_box::BoundingBox;
use crate::physics::{player_entity::PlayerEntity, trigger_entity::TriggerEntity};
use chariot_core::lap_info::*;
use glam::DVec3;

#[derive(Clone, Copy)]
pub struct Checkpoint {
    pub id: CheckpointID,
    pub bounds: BoundingBox,
}

impl Checkpoint {
    pub fn new(id: CheckpointID, bounds: BoundingBox) -> Self {
        Self { id, bounds }
    }
}

impl TriggerEntity for Checkpoint {
    fn pos(&self) -> DVec3 {
        self.bounds.pos()
    }

    fn get_bounding_box(&self) -> BoundingBox {
        self.bounds
    }

    fn trigger(&self, ply: &mut PlayerEntity) {
        ply.lap_info.last_checkpoint = self.id;
    }
}

#[derive(Clone, Copy)]
pub struct Zone {
    pub id: ZoneID,
    pub bounds: BoundingBox,
}

impl Zone {
    pub fn new(id: ZoneID, bounds: BoundingBox) -> Self {
        Self { id, bounds }
    }
}

impl TriggerEntity for Zone {
    fn pos(&self) -> DVec3 {
        self.bounds.pos()
    }

    fn get_bounding_box(&self) -> BoundingBox {
        self.bounds
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
    pub bounds: BoundingBox,
}

impl FinishLine {
    pub fn new(bounds: BoundingBox, last_zone: ZoneID) -> Self {
        Self { last_zone, bounds }
    }
}

impl TriggerEntity for FinishLine {
    fn pos(&self) -> DVec3 {
        self.bounds.pos()
    }

    fn get_bounding_box(&self) -> BoundingBox {
        self.bounds
    }

    fn trigger(&self, ply: &mut PlayerEntity) {
        println!("INSIDE");
        // Player is only allowed to advance if they are on the track's last zone
        if ply.lap_info.zone == self.last_zone {
            ply.lap_info.zone = 0;
        }
    }
}
