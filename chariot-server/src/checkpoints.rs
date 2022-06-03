use crate::physics::bounding_box::BoundingBox;
use crate::physics::{player_entity::PlayerEntity, trigger_entity::TriggerEntity};
use chariot_core::player::lap_info::*;
use chariot_core::GLOBAL_CONFIG;
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

    fn trigger(&mut self, player: &mut PlayerEntity) {
        player.lap_info.last_checkpoint = self.id;
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

    fn trigger(&mut self, player: &mut PlayerEntity) {
        // Only advance zone if the player is in the zone before us
        if (player.lap_info.zone + 1) == self.id {
            player.lap_info.zone = self.id;
            println!("Player now in zone {}", self.id);
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

    pub fn set_last_zone(&mut self, last_zone: ZoneID) -> Self {
        self.last_zone = last_zone;
        *self
    }
}

impl TriggerEntity for FinishLine {
    fn pos(&self) -> DVec3 {
        self.bounds.pos()
    }

    fn get_bounding_box(&self) -> BoundingBox {
        self.bounds
    }

    fn trigger(&mut self, player: &mut PlayerEntity) {
        // Player is only allowed to advance if they are on the track's last zone
        if player.lap_info.zone == self.last_zone {
            if player.lap_info.lap == GLOBAL_CONFIG.number_laps {
                player.lap_info.finished = true;
                println!("Player has finished {}!", player.lap_info.placement);
            } else {
                player.lap_info.lap += 1;
                player.lap_info.zone = 0;
                println!("Player now on lap {}", player.lap_info.lap);
            }
        }
    }
}
