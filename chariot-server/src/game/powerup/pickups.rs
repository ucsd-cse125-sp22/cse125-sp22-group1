use std::time::{Duration, Instant};

use chariot_core::GLOBAL_CONFIG;
use glam::DVec3;

use crate::physics::{
    bounding_box::BoundingBox, player_entity::PlayerEntity, trigger_entity::TriggerEntity,
};

impl PlayerEntity {
    pub fn give_powerup(&mut self) {
        if !self.has_powerup() {
            // Give a powerup
        }
    }

    pub fn has_powerup(&self) -> bool {
        match self.current_powerup {
            Some(_) => true,
            None => false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct PowerUpTrigger {
    pub bounds: BoundingBox,
    pub active_after: Instant,
}

impl PowerUpTrigger {
    pub fn new(bounds: BoundingBox) -> Self {
        Self {
            bounds,
            active_after: Instant::now(),
        }
    }
}

impl TriggerEntity for PowerUpTrigger {
    fn pos(&self) -> DVec3 {
        self.bounds.pos()
    }

    fn get_bounding_box(&self) -> BoundingBox {
        self.bounds
    }

    fn trigger(&mut self, player: &mut PlayerEntity) {
        // Player is only allowed to pick up if we are active
        if !player.has_powerup() && Instant::now() > self.active_after {
            player.give_powerup();
            self.active_after =
                Instant::now() + Duration::from_secs(GLOBAL_CONFIG.powerup_cooldown_time);
        }
    }
}
