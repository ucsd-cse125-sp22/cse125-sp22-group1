use crate::physics::bounding_box::BoundingBox;
use crate::physics::player_entity::PlayerEntity;

pub trait TriggerEntity {
    fn get_bounding_box(&self) -> BoundingBox;
    fn trigger(&self, colliding_player: &mut PlayerEntity);
}
