use chariot_core::physics_object::PhysicsProperties;
use chariot_core::physics_object::Vec3D;
use chariot_core::physics_object::magnitude_Vec3D;
use chariot_core::physics_object::EngineStatus;

mod constants;

/* Given a set of physical properties, compute and return what next tick's
 * physics properties will be for that object */
pub fn do_physics_step(previous_props: &PhysicsProperties, time_step: f64) -> PhysicsProperties{
	let forces = sum_of_forces_on_object(previous_props);
	let acceleration = forces / previous_props.mass;

	let velocity = previous_props.linear_momentum / previous_props.mass;

	return PhysicsProperties {
		position: previous_props.position + velocity * time_step,
		velocity: previous_props.velocity + acceleration * time_step,
		linear_momentum: previous_props.linear_momentum + forces * time_step,
		angular_momentum: previous_props.angular_momentum,
		mass: previous_props.mass,
		engine_status: previous_props.engine_status,
		unit_steer_direction: previous_props.unit_steer_direction,
	};
}


pub fn sum_of_forces_on_object(object: &PhysicsProperties) -> Vec3D
{
	return gravitational_force_on_object(object) +
		normal_force_on_object(object) +
		tractive_force_on_object(object) +
		air_resistance_force_on_object(object) +
		rolling_resistance_force_on_object(object) +
		braking_resistance_force_on_object(object);
}


fn gravitational_force_on_object(object: &PhysicsProperties) -> Vec3D {
	return Vec3D {x:0.0, y:-1.0, z:0.0} * object.mass * constants::GRAVITY_COEFFICIENT;
}

// unjustified temporary assumption we'll invalidate later: we're always on flat
// ground (otherwise, there's a horizontal component to normal force)
fn normal_force_on_object(object: &PhysicsProperties) -> Vec3D {
	return Vec3D {x:0.0, y:1.0, z:0.0} * object.mass;
}

// tractive force is what's applied by the "engine" == player-applied motive
// force forwards
fn tractive_force_on_object(object: &PhysicsProperties) -> Vec3D {
	match &object.engine_status {
		EngineStatus::ACCELERATING => return object.unit_steer_direction * object.mass * constants::CAR_ACCELERATOR,
		EngineStatus::NEUTRAL => return Vec3D {x:0.0, y:0.0, z:0.0},
		EngineStatus::BRAKING => return Vec3D {x:0.0, y:0.0, z:0.0},
	}
}

fn air_resistance_force_on_object(object: &PhysicsProperties) -> Vec3D
{
	// air resistance is proportion to the square of velocity
	return object.velocity * -1.0 * constants::DRAG_COEFFICIENT * magnitude_Vec3D(&object.velocity);
}

fn rolling_resistance_force_on_object(object: &PhysicsProperties) -> Vec3D
{
	return object.velocity * -1.0 * constants::ROLLING_RESISTANCE_COEFFICIENT;
}

fn braking_resistance_force_on_object(object: &PhysicsProperties) -> Vec3D {
	match &object.engine_status {
		EngineStatus::ACCELERATING => return Vec3D {x:0.0, y:0.0, z:0.0},
		EngineStatus::NEUTRAL => return Vec3D {x:0.0, y:0.0, z:0.0},
		EngineStatus::BRAKING => return object.unit_steer_direction * -1.0 * object.mass * constants::CAR_BRAKE,
	}
}
