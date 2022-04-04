extern crate chariot_core;
use chariot_core::physics_object::PhysicsProperties;
use chariot_core::physics_object::Vec3D;

/* Given a set of physical properties, compute and return what next tick's
 * physics properties will be for that object */
pub fn do_physics_step(previous_props: &PhysicsProperties, time_step: f64) -> PhysicsProperties{
	let forces = sum_of_forces_on_object(previous_props);
	let velocity = previous_props.linear_momentum / previous_props.mass;

	return PhysicsProperties {
		position: previous_props.position + velocity * time_step,
		linear_momentum: previous_props.linear_momentum + forces * time_step,
		angular_momentum: previous_props.angular_momentum,
		mass: previous_props.mass
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

fn gravitational_force_on_object(object: &PhysicsProperties) -> Vec3D { return Vec3D {x:0.0, y:0.0, z:0.0}; }

fn normal_force_on_object(object: &PhysicsProperties) -> Vec3D { return Vec3D {x:0.0, y:0.0, z:0.0}; }

fn tractive_force_on_object(object: &PhysicsProperties) -> Vec3D { return Vec3D {x:0.0, y:0.0, z:0.0}; }

fn air_resistance_force_on_object(object: &PhysicsProperties) -> Vec3D { return Vec3D {x:0.0, y:0.0, z:0.0}; }

fn rolling_resistance_force_on_object(object: &PhysicsProperties) -> Vec3D { return Vec3D {x:0.0, y:0.0, z:0.0}; }

fn braking_resistance_force_on_object(object: &PhysicsProperties) -> Vec3D { return Vec3D {x:0.0, y:0.0, z:0.0}; }