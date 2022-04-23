use crate::drawable::*;
use glam::Vec3;
use std::any::{Any, TypeId};
use std::any::{Any, TypeId};
use std::boxed::Box;

struct World {
    entity_count: usize,
    entities: Vec<Box<Entity>>,
}

struct Entity {
    components: HashMap<TypeId, Box<dyn Component>>,
    children: Vec<Box<Entity>>,
}

pub trait Component {
    fn as_any(&self) -> &dyn Any;
}

struct Transform {
    position: Vec3,
    rotation: Vec3,
    scale: Vec3,
}

impl Transform {
    fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ZERO,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}

impl Component for Transform {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct PlayerController {}

impl PlayerController {
    fn new() -> Self {
        Self {}
    }
}

impl Component for PlayerController {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Component for StaticMeshDrawable {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Component for Vec<StaticMeshDrawable> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Component for StaticMeshDrawable {
    fn update(&mut self) {
        //render entity
    }
}

impl Component for Camera {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Entity {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            children: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        let map = &mut self.components;
        for (key, mut component) in map {
            component.update();
        }
    }

    pub fn add_component<T: Component>(&mut self, component: T)
    where
        T: Component + 'static,
    {
        self.components
            .insert(TypeId::of::<T>(), Box::new(component));
    }

    pub fn get_component<T: Component>(&self) -> Option<&T>
    where
        T: Component + 'static,
    {
        let id = TypeId::of::<T>();
        self.components
            .get(&id)
            .map(|c| c.as_any().downcast_ref::<T>().unwrap())
    }

    pub fn add_child(&mut self, entity: Entity) -> usize {
        let id = self.children.len();
        self.children.push(Box::new(entity));
        id
    }

    pub fn get_child(&self, id: usize) -> &Entity {
        return self.children.get(id).unwrap();
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            entity_count: 0,
            entities: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Entity) -> usize {
        let id = self.entity_count;
        self.entities.push(Box::new(entity));
        self.entity_count += 1;
        id
    }

    pub fn get_entity(&self, id: usize) -> &Entity {
        return self.entities.get(id).unwrap();
    }
}

#[test]
fn test_entity_child() {
    let mut world = World::new();
    let mut entity = Entity::new();
    let mut child = Entity::new();
    let child_idx = entity.add_child(child);
    let entity_idx = world.add_entity(entity);
    world.get_entity(entity_idx).get_child(child_idx);
}
