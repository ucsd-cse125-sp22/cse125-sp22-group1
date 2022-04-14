use glam::Vec3;
use std::collections::HashMap;
use std::boxed::Box;
use std::any::{TypeId,Any};

struct World {
	entities: Vec<Box<Entity>>
}

struct Entity {
	components: HashMap<TypeId, Box<dyn Component>>
}

trait Component {
	fn update(&mut self);
	fn as_any(&self) -> &dyn Any;
}

struct Transform {
	position: Vec3,
	rotation: Vec3,
	scale: Vec3,
}

struct PlayerController {
	
}

struct StaticMeshDrawable {

}

impl Component for Transform {
	fn update(&mut self){
		//receive server data and update fields
	}
	
	fn as_any(&self) -> &dyn Any {
		self
	}
}

impl Component for PlayerController {
	fn update(&mut self){
		//send input data to server
	}
	
	fn as_any(&self) -> &dyn Any {
		self
	}
}

impl Component for StaticMeshDrawable {
	fn update(&mut self){
		//render entity
	}
	
	fn as_any(&self) -> &dyn Any {
		self
	}
}

impl Entity {
	
	fn update(&mut self){
		let map = &mut self.components;
		for (key, mut component) in map{
			component.update();
		}
	}

	fn add_component<T: Component>(&mut self, component: T)
		where
			T: Component + 'static,
	{
		self.components.insert(TypeId::of::<T>(), Box::new(component));
	}
	
	fn get_component<T: Component>(&self) -> Option<&T>
		where
			T: Component + 'static,
	{
		let id = TypeId::of::<T>();
		self.components
			.get(&id)
			.map(|c| c.as_any().downcast_ref::<T>().unwrap())
		
	}
}

fn main(){

}