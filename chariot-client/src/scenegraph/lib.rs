struct World {

}

struct Entity {
	components: Vec<dyn Component>,
}

trait Component {
	fn get_name(&self) -> &str;
	fn update(&self);
}

struct Rigidbody {
	position: {f32, f32},
	velocity: {f32, f32},
	acceleration: {f32, f32}
}

impl Component for Rigidbody {
	fn update(&self){
		self.velocity += self.acceleration;
		self.position += self.velocity;
	}
}

impl Entity {
	fn update(&self){
		for component in self.components.iter_mut(){
			component.update();
		}
	}
	fn add_component(&self, component: dyn Component){
		self.components.push(component);
	}
	fn get_component(&self, name: &str) -> dyn Component{
		for component in self.components {
			if component.get_name() == &str {
				return component;
			}
		}
	}
}

fn main(){

}