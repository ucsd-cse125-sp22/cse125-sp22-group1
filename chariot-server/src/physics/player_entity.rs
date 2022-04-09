use chariot_core::entity_location::EntityLocation;
use chariot_core::player_inputs::PlayerInputs;
use glam::DVec3;

pub struct PlayerEntity {
    pub velocity: DVec3,
    pub angular_velocity: f64, // in radians per time unit
    pub mass: f64,

    pub player_inputs: PlayerInputs,
    pub entity_location: EntityLocation,
}