use crate::physics::player_entity::PlayerEntity;

use super::PowerUp;

impl PowerUp {
    pub fn _activate(&self, _activator_id: usize, _players: &[PlayerEntity; 4]) {
        match self {
            _ => todo!(),
        }
    }
    pub fn _deactivate(&self, _activator_id: usize, _players: &[PlayerEntity; 4]) {
        match self {
            _ => todo!(),
        }
    }
}
