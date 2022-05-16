use crate::physics::player_entity::PlayerEntity;

use super::PowerUp;

impl PowerUp {
    pub fn activate(&self, activator_id: usize, players: &[PlayerEntity; 4]) {
        match self {
            _ => todo!(),
        }
    }
    pub fn deactivate(&self, activator_id: usize, players: &[PlayerEntity; 4]) {
        match self {
            _ => todo!(),
        }
    }
}
