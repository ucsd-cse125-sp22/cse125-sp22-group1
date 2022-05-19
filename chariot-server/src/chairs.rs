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

fn get_mass_from_chair_name(chair: &Chair) -> f64 {
    match chair {
        Chair::Standard => 10.0,
    }
}

fn get_size_from_chair_name(chair: &Chair) -> DVec3 {
    match chair {
        Chair::Standard => DVec3::new(1.0, 2.0, 1.0),
    }
}

// These numbers are completely random guesses btw
fn get_starting_position_from_player_number(player_number: PlayerID) -> DVec3 {
    return DVec3::new(0.0, 1.0, 20.0 * (1.5 - player_number as f64));
}

// Get the initial physics properties of a player (i.e. at the race start, before anyone starts going)
pub fn get_player_start_physics_properties(
    chair_name: &Chair,
    player_number: PlayerID,
) -> PlayerEntity {
    return PlayerEntity {
        velocity: DVec3::ZERO,
        angular_velocity: 0.0,
        mass: get_mass_from_chair_name(chair_name),
        size: get_size_from_chair_name(chair_name),
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
        lap_info: LapInformation::new(),
        current_powerup: None,
    };
}
