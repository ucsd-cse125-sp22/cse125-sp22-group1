use crate::drawable::*;
use crate::resources::{accum_bounds, new_bounds, Bounds};
use std::any::{Any, TypeId};
use std::boxed::Box;
use std::collections::HashMap;

pub struct World {
    root: Entity,
}

impl World {
    pub fn new() -> Self {
        Self {
            root: Entity::new(),
        }
    }

    pub fn root(&self) -> &Entity {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut Entity {
        &mut self.root
    }
}

pub struct Entity {
    components: HashMap<TypeId, Box<dyn Component>>,
    children: Vec<Entity>,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            children: Vec::new(),
        }
    }

    pub fn set_component<T: Component>(&mut self, component: T)
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
        self.children.push(entity);
        id
    }

    pub fn get_child(&self, id: usize) -> &Entity {
        return self.children.get(id).unwrap();
    }

    pub fn calc_bounds(&self) -> Bounds {
        let mut bounds = new_bounds();
        dfs(self, |e| {
            let cur_bounds = e
                .get_component::<Bounds>()
                .unwrap_or(&(glam::Vec3::ZERO, glam::Vec3::ZERO));
            let cur_transform = e
                .get_component::<Transform>()
                .unwrap_or(&Transform::default())
                .to_mat4();
            let transformed_bounds = (
                cur_transform.transform_point3(cur_bounds.0),
                cur_transform.transform_point3(cur_bounds.1),
            );
            bounds = accum_bounds(bounds, transformed_bounds);
        });

        bounds
    }
}

pub trait Component {
    fn as_any(&self) -> &dyn Any;
}

#[derive(Clone, Copy)]
pub struct Transform {
    pub translation: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Transform {
    pub fn to_mat4(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
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

pub struct Camera {
    pub orbit_angle: glam::Vec2,
    pub distance: f32,
}

impl Camera {
    pub fn view_mat4(&self) -> glam::Mat4 {
        let look_rot = glam::Quat::from_euler(
            glam::EulerRot::YXZ,
            self.orbit_angle.x,
            std::f32::consts::PI - self.orbit_angle.y,
            0.0,
        );
        let look_dir = look_rot * glam::Vec3::Z;

        let look_offset = look_dir * self.distance;

        glam::Mat4::look_at_rh(look_offset, glam::Vec3::ZERO, glam::Vec3::Y)
    }
}

impl Component for Camera {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct Light {
    pub view: glam::Mat4,
    pub proj: glam::Mat4,
    pub framebuffer_name: String,
}

impl Light {
    pub fn new_directional(dir: glam::Vec3, bounds: Bounds) -> Self {
        let scene_center = (bounds.0 + bounds.1) * 0.5;
        let scene_radius = (bounds.1 - scene_center).length();

        let dist_padding = 10.0;

        let light_pos = scene_center - dir * (scene_radius + dist_padding);
        let view = glam::Mat4::look_at_rh(light_pos, scene_center, glam::Vec3::Y);
        let proj = glam::Mat4::orthographic_rh(
            -scene_radius,
            scene_radius,
            -scene_radius,
            scene_radius,
            0.01,
            1000.0,
        );

        Self {
            view,
            proj,
            framebuffer_name: "shadow_out1".to_string(),
        }
    }
}

impl Component for Light {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Component for Bounds {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn dfs<'a, F>(root: &'a Entity, mut func: F)
where
    F: FnMut(&'a Entity),
{
    let mut entity_stack = vec![root];
    while !entity_stack.is_empty() {
        let cur_entity = entity_stack.pop().unwrap();
        func(cur_entity);

        entity_stack.extend(cur_entity.children.iter());
    }
}

pub fn dfs_mut<'a, F>(root: &mut Entity, func: &F)
where
    F: Fn(&mut Entity),
{
    for child in root.children.iter_mut() {
        func(child);
        dfs_mut(child, func);
    }
}

pub fn dfs_acc<'a, T, F>(root: &'a Entity, acc_init: T, mut func: F)
where
    F: FnMut(&'a Entity, &T) -> T,
{
    let mut entity_stack = vec![root];
    let mut acc_stack = vec![acc_init];
    while !entity_stack.is_empty() {
        let cur_entity = entity_stack.pop().unwrap();
        let cur_acc = acc_stack.pop().unwrap();

        for child in cur_entity.children.iter() {
            let acc = func(child, &cur_acc);
            acc_stack.push(acc);
            entity_stack.push(child);
        }
    }
}

#[test]
fn test_entity_child() {
    let mut world = World::new();
    let mut entity = Entity::new();
    let mut child = Entity::new();
    let child_idx = entity.add_child(child);
    //let entity_idx = world.add_entity(entity);
    //world.get_entity(entity_idx).get_child(child_idx);
}
