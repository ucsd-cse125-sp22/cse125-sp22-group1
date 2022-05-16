use glam::DVec3;

use crate::physics::bounding_box::BoundingBox;
use crate::physics::player_entity::PlayerEntity;

pub trait TriggerEntity {
    fn pos(&self) -> DVec3;
    fn get_bounding_box(&self) -> BoundingBox;
    fn trigger(&mut self, colliding_player: &mut PlayerEntity);
}
