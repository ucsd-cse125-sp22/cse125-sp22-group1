use glam::DVec3;

use chariot_core::entity_location::EntityLocation;
use chariot_core::GLOBAL_CONFIG;
use chariot_core::lap_info::LapInformation;
use chariot_core::player_inputs::EngineStatus;
use chariot_core::player_inputs::PlayerInputs;
use chariot_core::player_inputs::RotationStatus;

use crate::physics::player_entity::PlayerEntity;

#[test]
fn test_accelerating() {
    let mut props = PlayerEntity {
        player_inputs: PlayerInputs {
            engine_status: EngineStatus::Accelerating,
            rotation_status: RotationStatus::NotInSpin,
        },

        entity_location: EntityLocation {
            position: DVec3::new(0.0, 0.0, 0.0),
            unit_steer_direction: DVec3::new(0.6, 0.0, 0.8),
            unit_upward_direction: DVec3::new(0.0, 1.0, 0.0),
        },

        velocity: DVec3::new(2.0, 0.0, 1.0),
        angular_velocity: 0.0,
        mass: 10.0,

        size: DVec3::new(10.0, 10.0, 10.0),
        bounding_box: [[-5.0, 5.0], [-5.0, 5.0], [-5.0, 5.0]],
        physics_changes: Vec::new(),
        lap_info: LapInformation::new(),
    };

    props = props.do_physics_step(1.0, Vec::new(), Vec::new());

    // since we're accelerating, should have the following changes:
    // - should have moved forward by previous velocity times time step
    assert!(props
        .entity_location
        .position
        .abs_diff_eq(DVec3::new(2.0, 0.0, 1.0), 0.001));
    // - velocity should have increased by acceleration amount in steer
    // direction, and decreased because of drag and rolling resistance
    let expected_velocity = DVec3::new(2.0, 0.0, 1.0)
        + DVec3::new(0.6, 0.0, 0.8) * GLOBAL_CONFIG.car_accelerator
        + DVec3::new(-2.0, 0.0, -1.0) * GLOBAL_CONFIG.drag_coefficient * (5.0 as f64).sqrt()
        + DVec3::new(-2.0, 0.0, -1.0) * GLOBAL_CONFIG.rolling_resistance_coefficient;
    assert!(props.velocity.abs_diff_eq(
        expected_velocity.normalize() * GLOBAL_CONFIG.max_car_speed,
        0.001
    ));
}

#[test]
fn test_non_accelerating() {
    let mut props = PlayerEntity {
        player_inputs: PlayerInputs {
            engine_status: EngineStatus::Neutral,
            rotation_status: RotationStatus::NotInSpin,
        },

        entity_location: EntityLocation {
            position: DVec3::new(0.0, 0.0, 0.0),
            unit_steer_direction: DVec3::new(0.6, 0.0, 0.8),
            unit_upward_direction: DVec3::new(0.0, 1.0, 0.0),
        },

        velocity: DVec3::new(2.0, 0.0, 1.0),
        angular_velocity: 0.0,
        mass: 10.0,

        size: DVec3::new(10.0, 10.0, 10.0),
        bounding_box: [[15.0, 25.0], [25.0, 35.0], [35.0, 45.0]],
        physics_changes: Vec::new(),
        lap_info: LapInformation::new(),
    };

    props = props.do_physics_step(1.0, Vec::new(), Vec::new());

    // since we're not accelerating, should have the following changes:
    // - should have moved forward by previous velocity times time step
    assert!(props
        .entity_location
        .position
        .abs_diff_eq(DVec3::new(2.0, 0.0, 1.0), 0.001));
    // - velocity should only have decreased, due to drag and rolling resistance
    let expected_velocity = DVec3::new(2.0, 0.0, 1.0)
        + DVec3::new(-2.0, 0.0, -1.0) * GLOBAL_CONFIG.drag_coefficient * (5.0 as f64).sqrt()
        + DVec3::new(-2.0, 0.0, -1.0) * GLOBAL_CONFIG.rolling_resistance_coefficient;
    assert!(props.velocity.abs_diff_eq(
        expected_velocity.normalize() * GLOBAL_CONFIG.max_car_speed,
        0.001
    ));
}

#[test]
fn test_decelerating() {
    let mut props = PlayerEntity {
        player_inputs: PlayerInputs {
            engine_status: EngineStatus::Braking,
            rotation_status: RotationStatus::NotInSpin,
        },

        entity_location: EntityLocation {
            position: DVec3::new(0.0, 0.0, 0.0),
            unit_steer_direction: DVec3::new(0.6, 0.0, 0.8),
            unit_upward_direction: DVec3::new(0.0, 1.0, 0.0),
        },

        velocity: DVec3::new(2.0, 0.0, 1.0),
        angular_velocity: 0.0,
        mass: 10.0,

        size: DVec3::new(10.0, 10.0, 10.0),
        bounding_box: [[15.0, 25.0], [25.0, 35.0], [35.0, 45.0]],
        physics_changes: Vec::new(),
        lap_info: LapInformation::new(),
    };

    props = props.do_physics_step(1.0, Vec::new(), Vec::new());

    // since we're decelerating, should have the following changes:
    // - should have moved forward by previous velocity times time step
    assert!(props
        .entity_location
        .position
        .abs_diff_eq(DVec3::new(2.0, 0.0, 1.0), 0.001));
    // - velocity should only have decreased, due to braking, drag, and rolling resistance
    let prev_velocity = DVec3::new(2.0, 0.0, 1.0);
    let neg_prev_velocity = DVec3::new(-2.0, 0.0, -1.0);
    let expected_velocity = prev_velocity
        + (neg_prev_velocity / neg_prev_velocity.length()) * GLOBAL_CONFIG.car_brake
        + neg_prev_velocity * GLOBAL_CONFIG.drag_coefficient * (5.0 as f64).sqrt()
        + neg_prev_velocity * GLOBAL_CONFIG.rolling_resistance_coefficient;
    assert!(props.velocity.abs_diff_eq(
        expected_velocity.normalize() * GLOBAL_CONFIG.max_car_speed,
        0.001
    ));
}

#[test]
fn test_spinning() {
    let mut props = PlayerEntity {
        player_inputs: PlayerInputs {
            engine_status: EngineStatus::Braking,
            rotation_status: RotationStatus::InSpinClockwise,
        },

        entity_location: EntityLocation {
            position: DVec3::new(0.0, 0.0, 0.0),
            unit_steer_direction: DVec3::new(0.6, 0.0, 0.8),
            unit_upward_direction: DVec3::new(0.0, 1.0, 0.0),
        },

        velocity: DVec3::new(0.0, 0.0, 0.0),
        angular_velocity: 0.0,
        mass: 10.0,

        size: DVec3::new(10.0, 10.0, 10.0),
        bounding_box: [[15.0, 25.0], [25.0, 35.0], [35.0, 45.0]],
        physics_changes: Vec::new(),
        lap_info: LapInformation::new(),
    };

    props = props.do_physics_step(1.0, Vec::new(), Vec::new());

    assert_eq!(props.angular_velocity, GLOBAL_CONFIG.car_spin);

    props.player_inputs.rotation_status = RotationStatus::NotInSpin;
    props = props.do_physics_step(1.0, Vec::new(), Vec::new());

    assert_eq!(
        props.angular_velocity,
        GLOBAL_CONFIG.car_spin * GLOBAL_CONFIG.rotation_reduction_coefficient
    );
}
