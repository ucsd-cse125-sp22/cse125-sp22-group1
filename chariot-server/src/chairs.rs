use crate::physics::bounding_box::BoundingBox;
use chariot_core::{
    entity_location::EntityLocation,
    player::{
        choices::Chair,
        lap_info::LapInformation,
        player_inputs::{EngineStatus, PlayerInputs, RotationStatus},
        PlayerID,
    },
};
use glam::DVec3;

use crate::physics::player_entity::PlayerEntity;

// We could implement something to load mass and size from a file or whatever,
// but it's probably just fine to hard-code them in here

// These numbers are completely random guesses btw
fn get_starting_position_from_player_number(player_number: PlayerID) -> DVec3 {
    return DVec3::new(21.5 + 2.0 * (1.5 - player_number as f64), 1.0, 65.0);
}

// Get the initial physics properties of a player (i.e. at the race start, before anyone starts going)
pub fn get_player_start_physics_properties(chair: &Chair, player_number: PlayerID) -> PlayerEntity {
    return PlayerEntity {
        velocity: DVec3::ZERO,
        angular_velocity: 0.0,
        size: chair.scale(),
        bounding_box: BoundingBox::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0), // will be made correct on the first physics tick
        player_inputs: PlayerInputs {
            engine_status: EngineStatus::Neutral,
            rotation_status: RotationStatus::NotInSpin,
        },
        entity_location: EntityLocation {
            position: get_starting_position_from_player_number(player_number),
            unit_steer_direction: DVec3::Z,
            unit_upward_direction: DVec3::Y,
        },
        physics_changes: vec![],
        stats_changes: vec![],
        current_colliders: vec![],
        sound_effects: vec![],
        lap_info: LapInformation::new(),
        current_powerup: None,
        chair: *chair,
    };
}
