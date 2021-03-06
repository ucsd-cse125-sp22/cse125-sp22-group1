use std::time::Instant;

use chariot_core::player::choices::Chair;
use glam::DVec3;

use crate::physics::bounding_box::BoundingBox;
use crate::physics::ramp::RampCollisionResult;
use crate::progress::PlayerProgress;
use chariot_core::entity_location::EntityLocation;
use chariot_core::player::{
    lap_info::LapInformation,
    player_inputs::{EngineStatus, PlayerInputs, RotationStatus},
};
use chariot_core::GLOBAL_CONFIG;

use crate::physics::player_entity::PlayerEntity;

fn get_starting_player_props() -> PlayerEntity {
    PlayerEntity {
        player_inputs: PlayerInputs {
            engine_status: EngineStatus::Accelerating(1.0),
            rotation_status: RotationStatus::NotInSpin,
        },

        entity_location: EntityLocation {
            position: DVec3::ZERO,
            unit_steer_direction: DVec3::new(0.6, 0.0, 0.8),
            unit_upward_direction: DVec3::Y,
        },

        velocity: DVec3::new(2.0, 0.0, 1.0),
        angular_velocity: 0.0,

        current_colliders: Vec::new(),

        size: DVec3::new(10.0, 10.0, 10.0),
        bounding_box: BoundingBox::new(-5.0, 5.0, -5.0, 5.0, -5.0, 5.0),
        physics_changes: Vec::new(),
        stats_changes: Vec::new(),
        sound_effects: Vec::new(),
        placement_data: PlayerProgress::PreGame,
        cached_place: None,
        current_powerup: None,
        chair: Chair::Swivel,
        game_start_time: Instant::now(),
    }
}

fn get_origin_cube() -> PlayerEntity {
    PlayerEntity {
        player_inputs: PlayerInputs {
            engine_status: EngineStatus::Neutral,
            rotation_status: RotationStatus::NotInSpin,
        },

        entity_location: EntityLocation {
            position: DVec3::ZERO,
            unit_steer_direction: DVec3::X,
            unit_upward_direction: DVec3::Y,
        },

        velocity: DVec3::ZERO,
        angular_velocity: 0.0,

        current_colliders: Vec::new(),

        size: DVec3::new(10.0, 10.0, 10.0),
        bounding_box: BoundingBox::new(-5.0, 5.0, -5.0, 5.0, -5.0, 5.0),
        physics_changes: Vec::new(),
        stats_changes: Vec::new(),
        sound_effects: Vec::new(),
        placement_data: PlayerProgress::PreGame,
        cached_place: None,
        current_powerup: None,
        chair: Chair::Swivel,
        game_start_time: Instant::now(),
    }
}

#[test]
fn test_spinning() {
    let mut props = get_starting_player_props();
    props.velocity = DVec3::ZERO;
    props.player_inputs.rotation_status = RotationStatus::InSpinClockwise(1.0);
    props = props.do_physics_step(
        1.0,
        Vec::new(),
        Vec::new(),
        std::iter::empty(),
        &Vec::new(),
        &RampCollisionResult::NoEffect,
    );

    assert_eq!(props.angular_velocity, GLOBAL_CONFIG.car_spin);

    props.player_inputs.rotation_status = RotationStatus::NotInSpin;
    props = props.do_physics_step(
        1.0,
        Vec::new(),
        Vec::new(),
        std::iter::empty(),
        &Vec::new(),
        &RampCollisionResult::NoEffect,
    );

    assert_eq!(
        props.angular_velocity,
        GLOBAL_CONFIG.car_spin * GLOBAL_CONFIG.rotation_reduction_coefficient
    );
}

#[test]
fn test_collision_with_self() {
    let origin_cube = get_origin_cube();
    assert!(origin_cube
        .bounding_box
        .is_colliding(&origin_cube.bounding_box));
}

#[test]
fn test_engulfed_collision() {
    let big_origin_cube = get_origin_cube();
    let mut smol_origin_cube = get_origin_cube();
    smol_origin_cube.size = DVec3::new(1.0, 1.0, 1.0);
    smol_origin_cube.update_bounding_box();
    assert!(big_origin_cube
        .bounding_box
        .is_colliding(&smol_origin_cube.bounding_box));
}

#[test]
fn test_collision_on_corner() {
    let origin_cube = get_origin_cube();
    let mut not_origin_cube = get_origin_cube();
    not_origin_cube.entity_location.position = DVec3::new(10.0, 10.0, 10.0);
    not_origin_cube.update_bounding_box();
    assert!(origin_cube
        .bounding_box
        .is_colliding(&not_origin_cube.bounding_box));
}

#[test]
fn test_noncollision_on_corner() {
    let origin_cube = get_origin_cube();
    let mut not_origin_cube = get_origin_cube();
    not_origin_cube.entity_location.position = DVec3::new(10.1, 10.1, 10.1);
    not_origin_cube.update_bounding_box();
    assert!(!origin_cube
        .bounding_box
        .is_colliding(&not_origin_cube.bounding_box));
}

// we have different logic for the y-direction, might as well test that
#[test]
fn test_noncollision_when_above_or_below() {
    let origin_cube = get_origin_cube();
    let mut high_cube = get_origin_cube();
    let mut low_cube = get_origin_cube();
    high_cube.entity_location.position = DVec3::new(0.0, 20.0, 0.0);
    low_cube.entity_location.position = DVec3::new(0.0, -20.0, 0.0);
    high_cube.update_bounding_box();
    low_cube.update_bounding_box();
    assert!(!origin_cube
        .bounding_box
        .is_colliding(&high_cube.bounding_box));
    assert!(!origin_cube
        .bounding_box
        .is_colliding(&low_cube.bounding_box));
    assert!(!high_cube.bounding_box.is_colliding(&low_cube.bounding_box)); // just for good measure
}

#[test]
fn test_collision_when_above_or_below() {
    let origin_cube = get_origin_cube();
    let mut high_cube = get_origin_cube();
    let mut low_cube = get_origin_cube();
    high_cube.entity_location.position = DVec3::new(0.0, 8.0, 0.0);
    low_cube.entity_location.position = DVec3::new(0.0, -8.0, 0.0);
    high_cube.update_bounding_box();
    low_cube.update_bounding_box();
    assert!(origin_cube
        .bounding_box
        .is_colliding(&high_cube.bounding_box));
    assert!(origin_cube
        .bounding_box
        .is_colliding(&low_cube.bounding_box));
    assert!(!high_cube.bounding_box.is_colliding(&low_cube.bounding_box)); // just for good measure
}

#[test]
fn test_collision_on_rotated_edges() {
    // uwu w-wat if i was a cube with edge wength 10 (??? ???????????????????? ???) centewed
    // at the o-owigin and wotated 45 degwees (>??^)
    // a-and u (????????) wewe a cube with edge wength 10 a-awso wotated 45
    // degwees (o????????o) but c-centewed 10sqwt(2) units away in the
    // x-diwection Owo a-and we ???? ???? ???? t-touched edges ????
    let mut owo_cube = get_origin_cube();
    let mut uwu_cube = get_origin_cube();

    owo_cube.entity_location.unit_steer_direction =
        DVec3::new(2.0_f64.sqrt() / 2.0, 0.0, 2.0_f64.sqrt() / 2.0);
    uwu_cube.entity_location.unit_steer_direction =
        DVec3::new(2.0_f64.sqrt() / 2.0, 0.0, 2.0_f64.sqrt() / 2.0);

    uwu_cube.entity_location.position = DVec3::new(10.0 * 2.0_f64.sqrt() - 0.1, 0.0, 0.0);
    uwu_cube.update_bounding_box();
    owo_cube.update_bounding_box();
    assert!(uwu_cube.bounding_box.is_colliding(&owo_cube.bounding_box));
}

#[test]
fn test_collision_on_30_deg_rotated_edges() {
    // uwu owo yadda yadda
    let mut owo_cube = get_origin_cube();
    let mut uwu_cube = get_origin_cube();

    // upper right corner is located at x = 5sqrt(6) / 2, z = 5sqrt(2) / 2
    owo_cube.entity_location.unit_steer_direction =
        DVec3::new(1.0 / 2.0, 0.0, 3.0_f64.sqrt() / 2.0);
    uwu_cube.entity_location.unit_steer_direction =
        DVec3::new(-1.0 / 2.0, 0.0, 3.0_f64.sqrt() / 2.0);

    uwu_cube.entity_location.position = DVec3::new(5.0 * 6.0_f64.sqrt(), 0.0, 0.0);
    uwu_cube.update_bounding_box();
    owo_cube.update_bounding_box();
    assert!(uwu_cube.bounding_box.is_colliding(&owo_cube.bounding_box));
}

#[test]
fn test_3d_bounding_box() {
    let mut cube = get_origin_cube();

    cube.size = DVec3::new(1.0, 10000.0, 1.0);
    cube.entity_location.unit_upward_direction =
        DVec3::new(2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0, 0.0);
    cube.update_bounding_box();

    let BoundingBox { min_y, max_y, .. } = cube.bounding_box;

    let actual_top = (10_000.0 / 2.0) / (2.0_f64.sqrt());
    let actual_bottom = (-10_000.0 / 2.0) / (2.0_f64.sqrt());
    assert!(actual_top * 0.999 < max_y && max_y < actual_top * 1.001);
    assert!(actual_bottom * 0.999 > min_y && min_y > actual_bottom * 1.001);
}
