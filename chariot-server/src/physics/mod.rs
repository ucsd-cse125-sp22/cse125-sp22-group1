extern crate glam;

use glam::DVec3;

use chariot_core::physics_object::PhysicsProperties;
use chariot_core::physics_object::EngineStatus;
use chariot_core::physics_object::RotationStatus;

mod constants;

/* Given a set of physical properties, compute and return what next tick's
 * physics properties will be for that object */
pub fn do_physics_step(previous_props: &PhysicsProperties, time_step: f64) -> PhysicsProperties{
	let forces = sum_of_forces_on_object(previous_props);
	let acceleration = forces / previous_props.mass;

	let velocity = previous_props.linear_momentum / previous_props.mass;

	let angular_velocity: f64 = match previous_props.rotation_status {
		RotationStatus::InSpinClockwise => previous_props.angular_velocity + constants::CAR_SPIN,
		RotationStatus::InSpinCounterclockwise => previous_props.angular_velocity - constants::CAR_SPIN,
		RotationStatus::NotInSpin => previous_props.angular_velocity * constants::ROTATION_REDUCTION_COEFFICIENT,
	};

	return PhysicsProperties {
		position: previous_props.position + velocity * time_step,
		velocity: previous_props.velocity + acceleration * time_step,
		linear_momentum: previous_props.linear_momentum + forces * time_step,
		angular_velocity: angular_velocity,
		mass: previous_props.mass,
		engine_status: previous_props.engine_status,
		rotation_status: previous_props.rotation_status,
		unit_steer_direction: previous_props.unit_steer_direction,
	};
}


pub fn sum_of_forces_on_object(object: &PhysicsProperties) -> DVec3
{
	return gravitational_force_on_object(object) +
		normal_force_on_object(object) +
		tractive_force_on_object(object) +
		air_resistance_force_on_object(object) +
		rolling_resistance_force_on_object(object) +
		braking_resistance_force_on_object(object);
}


fn gravitational_force_on_object(object: &PhysicsProperties) -> DVec3 {
	return DVec3::new(0.0, -1.0, 0.0) * object.mass * constants::GRAVITY_COEFFICIENT;
}

// unjustified temporary assumption we'll invalidate later: we're always on flat
// ground (otherwise, there's a horizontal component to normal force)
fn normal_force_on_object(object: &PhysicsProperties) -> DVec3 {
	return DVec3::new(0.0, 1.0, 0.0) * object.mass;
}

// tractive force is what's applied by the "engine" == player-applied motive
// force forwards
fn tractive_force_on_object(object: &PhysicsProperties) -> DVec3 {
	match &object.engine_status {
		EngineStatus::Accelerating => return object.unit_steer_direction * object.mass * constants::CAR_ACCELERATOR,
		EngineStatus::Neutral => return DVec3::new(0.0, 0.0, 0.0),
		EngineStatus::Braking => return DVec3::new(0.0, 0.0, 0.0),
	}
}

fn air_resistance_force_on_object(object: &PhysicsProperties) -> DVec3
{
	// air resistance is proportional to the square of velocity
	return object.velocity * object.mass * -1.0 * constants::DRAG_COEFFICIENT * object.velocity.length();
}

fn rolling_resistance_force_on_object(object: &PhysicsProperties) -> DVec3
{
	return object.velocity * object.mass * -1.0 * constants::ROLLING_RESISTANCE_COEFFICIENT;
}

fn braking_resistance_force_on_object(object: &PhysicsProperties) -> DVec3 {
	match &object.engine_status {
		EngineStatus::Accelerating => return DVec3::new(0.0, 0.0, 0.0),
		EngineStatus::Neutral => return DVec3::new(0.0, 0.0, 0.0),
		// divide velocity by its magnitude to have a unit vector pointing
		// opposite current heading
		EngineStatus::Braking => return object.velocity / object.velocity.length() * -1.0 * object.mass * constants::CAR_BRAKE,
	}
}


#[test]
fn test_accelerating() {
	let mut props = PhysicsProperties {
		position: DVec3::new( 20.0,  30.0,  40.0),
		velocity: DVec3::new( 2.0,  0.0,  1.0),

		linear_momentum: DVec3::new( 20.0,  0.0,  10.0),

		mass: 10.0,

		unit_steer_direction: DVec3::new( 0.6,  0.0,  0.8),
		angular_velocity: 0.0,
		engine_status: EngineStatus::Accelerating,
		rotation_status: RotationStatus::NotInSpin,
	};

	props = do_physics_step(&props, 1.0);

	// since we're accelerating, should have the following changes:
	// - should have moved forward by previous velocity times time step
	assert!(props.position.abs_diff_eq(DVec3::new( 22.0,  30.0,  41.0), 0.001));
	// - velocity should have increased by acceleration amount in steer
	// direction, and decreased because of drag and rolling resistance
	let expected_velocity =
		DVec3::new( 2.0,  0.0,  1.0) +
		DVec3::new( 0.6,  0.0,  0.8) * constants::CAR_ACCELERATOR +
		DVec3::new( -2.0,  0.0,  -1.0) * constants::DRAG_COEFFICIENT * (5.0 as f64).sqrt()  +
		DVec3::new( -2.0,  0.0,  -1.0) * constants::ROLLING_RESISTANCE_COEFFICIENT;
	assert!(props.velocity.abs_diff_eq(expected_velocity, 0.001));
	// momentum is just mass times velocity
	assert!(props.linear_momentum.abs_diff_eq(expected_velocity * props.mass, 0.001));
}


#[test]
fn test_non_accelerating() {
	let mut props = PhysicsProperties {
		position: DVec3::new( 20.0,  30.0,  40.0),
		velocity: DVec3::new( 2.0,  0.0,  1.0),

		linear_momentum: DVec3::new( 20.0,  0.0,  10.0),

		mass: 10.0,

		unit_steer_direction: DVec3::new( 0.6,  0.0,  0.8),
		angular_velocity: 0.0,
		engine_status: EngineStatus::Neutral,
		rotation_status: RotationStatus::NotInSpin,
	};

	props = do_physics_step(&props, 1.0);

	// since we're not accelerating, should have the following changes:
	// - should have moved forward by previous velocity times time step
	assert!(props.position.abs_diff_eq(DVec3::new( 22.0,  30.0,  41.0), 0.001));
	// - velocity should only have decreased, due to drag and rolling resistance
	let expected_velocity =
		DVec3::new( 2.0,  0.0,  1.0) +
		DVec3::new( -2.0,  0.0,  -1.0) * constants::DRAG_COEFFICIENT * (5.0 as f64).sqrt()  +
		DVec3::new( -2.0,  0.0,  -1.0) * constants::ROLLING_RESISTANCE_COEFFICIENT;
	assert!(props.velocity.abs_diff_eq(expected_velocity, 0.001));
	// momentum is just mass times velocity
	assert!(props.linear_momentum.abs_diff_eq(expected_velocity * props.mass, 0.001));
}

#[test]
fn test_decelerating() {
	let mut props = PhysicsProperties {
		position: DVec3::new( 20.0,  30.0,  40.0),
		velocity: DVec3::new( 2.0,  0.0,  1.0),

		linear_momentum: DVec3::new( 20.0,  0.0,  10.0),

		mass: 10.0,

		unit_steer_direction: DVec3::new( 0.6,  0.0,  0.8),
		angular_velocity: 0.0,
		engine_status: EngineStatus::Braking,
		rotation_status: RotationStatus::NotInSpin,
	};

	props = do_physics_step(&props, 1.0);

	// since we're decelerating, should have the following changes:
	// - should have moved forward by previous velocity times time step
	assert!(props.position.abs_diff_eq(DVec3::new( 22.0,  30.0,  41.0), 0.001));
	// - velocity should only have decreased, due to braking, drag, and rolling resistance
	let prev_velocity = DVec3::new( 2.0,  0.0,  1.0);
	let neg_prev_velocity = DVec3::new( -2.0,  0.0,  -1.0);
	let expected_velocity =
		prev_velocity +
		(neg_prev_velocity / neg_prev_velocity.length()) * constants::CAR_BRAKE +
		neg_prev_velocity * constants::DRAG_COEFFICIENT * (5.0 as f64).sqrt()  +
		neg_prev_velocity * constants::ROLLING_RESISTANCE_COEFFICIENT;
	assert!(props.velocity.abs_diff_eq(expected_velocity, 0.001));
	// momentum is just mass times velocity
	assert!(props.linear_momentum.abs_diff_eq(expected_velocity * props.mass, 0.001));
}

#[test]
fn test_spinning() {
	let mut props = PhysicsProperties {
		position: DVec3::new( 20.0,  30.0,  40.0),
		velocity: DVec3::new( 0.0,  0.0,  0.0),

		linear_momentum: DVec3::new( 0.0,  0.0,  0.0),

		mass: 10.0,

		unit_steer_direction: DVec3::new( 0.6,  0.0,  0.8),
		angular_velocity: 0.0,
		engine_status: EngineStatus::Braking,
		rotation_status: RotationStatus::InSpinClockwise,
	};

	props = do_physics_step(&props, 1.0);
	assert_eq!(props.angular_velocity, constants::CAR_SPIN);

	props.rotation_status = RotationStatus::NotInSpin;
	props = do_physics_step(&props, 1.0);
	assert_eq!(props.angular_velocity, constants::CAR_SPIN * constants::ROTATION_REDUCTION_COEFFICIENT);
}