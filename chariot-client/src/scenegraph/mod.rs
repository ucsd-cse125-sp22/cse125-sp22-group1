use crate::drawable::*;
use crate::resources::{accum_bounds, new_bounds, Bounds};
use std::any::{Any, TypeId};
use std::borrow::Borrow;
use std::boxed::Box;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

pub type Entity = u32;

pub const NULL_ENTITY: Entity = 0;

pub trait Component: Sized + Default + 'static {
    type Storage: ComponentStorage<Self>;
}

impl<T: Sized + Default + 'static> Component for T {
    type Storage = VecStorage<T>;
}

// TODO: remove
pub trait ComponentStorage<T: Component>: Default {
    unsafe fn insert_unchecked(&mut self, entity: Entity, data: T);
    unsafe fn get_unchecked(&self, entity: Entity) -> &T;
    unsafe fn get_unchecked_mut(&mut self, entity: Entity) -> &mut T;
    fn contains(&self, entity: Entity) -> bool;
}

#[derive(Default)]
pub struct VecStorage<T: Component> {
    dense: Vec<T>,
    entities: Vec<Entity>,
    sparse: Vec<MaybeUninit<u32>>,
}

impl<T: Component> VecStorage<T> {
    pub fn iter(&self) -> std::slice::Iter<T> {
        self.dense.iter()
    }
}

impl<T: Component> ComponentStorage<T> for VecStorage<T> {
    unsafe fn insert_unchecked(&mut self, entity: Entity, data: T) {
        let eidx = entity as usize;
        if self.sparse.len() <= eidx {
            let delta = eidx + 1 - self.sparse.len();
            self.sparse.reserve(delta);
            self.sparse.set_len(eidx + 1);
        }

        let comp_idx = self.dense.len() as u32;

        self.sparse
            .get_unchecked_mut(eidx)
            .as_mut_ptr()
            .write(comp_idx);
        self.entities.push(entity);
        self.dense.push(data);
    }

    unsafe fn get_unchecked(&self, entity: Entity) -> &T {
        let eidx = entity as usize;
        let comp_idx = self.sparse.get_unchecked(eidx).assume_init() as usize;
        self.dense.get_unchecked(comp_idx)
    }

    unsafe fn get_unchecked_mut(&mut self, entity: Entity) -> &mut T {
        let eidx = entity as usize;
        let comp_idx = self.sparse.get_unchecked(eidx).assume_init() as usize;
        self.dense.get_unchecked_mut(comp_idx)
    }

    fn contains(&self, entity: Entity) -> bool {
        let eidx = entity as usize;
        self.sparse
            .get(eidx)
            .map(|cidx| {
                self.entities
                    .get(unsafe { cidx.assume_init() } as usize)
                    .map(|eidx2| eidx == (*eidx2 as usize))
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }
}

pub struct Builder<'a> {
    entity: Entity,
    world: &'a mut World,
}

impl<'a> Builder<'a> {
    pub fn with<T: Component>(&'a mut self, component: T) -> &'a mut Self {
        let type_id = TypeId::of::<T>();
        let comp_storage: &mut T::Storage = self
            .world
            .storage_mut::<T>()
            .expect("Attempting to add unregistered component type to entity");

        unsafe {
            comp_storage.insert_unchecked(self.entity, component);
        }

        self
    }

    pub fn attach(&'a mut self, parent: Entity) -> &'a mut Self {
        let parent_node_copy = self
            .world
            .get_mut::<SceneNode>(parent)
            .expect("Parent must have Scene Node component to attach child")
            .clone();

        let scene_node = SceneNode {
            first: NULL_ENTITY,
            next: parent_node_copy.first,
            prev: NULL_ENTITY,
            parent: parent,
        };

        let comp_storage: &mut <SceneNode as Component>::Storage = self
            .world
            .storage_mut::<SceneNode>()
            .expect("Attempting to add unregistered component type to entity");

        unsafe {
            comp_storage.insert_unchecked(self.entity, scene_node);
        }

        if parent_node_copy.first != NULL_ENTITY {
            unsafe { comp_storage.get_unchecked_mut(parent_node_copy.first) }.prev = self.entity;
        }

        unsafe { comp_storage.get_unchecked_mut(parent) }.first = self.entity;

        self
    }

    pub fn build(&self) -> Entity {
        self.entity
    }
}

trait Resource: Any + 'static {
    fn __get_type_id(&self) -> TypeId;
}

impl<T: Any> Resource for T {
    fn __get_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

impl dyn Resource {
    #[inline]
    pub fn is<T: Resource>(&self) -> bool {
        TypeId::of::<T>() == Resource::__get_type_id(self)
    }

    #[inline]
    pub fn downcast_ref<T: Resource>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { Option::Some(self.downcast_ref_unchecked()) }
        } else {
            Option::None
        }
    }

    #[inline]
    pub unsafe fn downcast_ref_unchecked<T: Resource>(&self) -> &T {
        &*(self as *const Self as *const T)
    }

    #[inline]
    pub fn downcast_mut<T: Resource>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe { Option::Some(self.downcast_mut_unchecked()) }
        } else {
            Option::None
        }
    }

    #[inline]
    pub unsafe fn downcast_mut_unchecked<T: Resource>(&mut self) -> &mut T {
        &mut *(self as *mut Self as *mut T)
    }
}

pub struct World {
    components: HashMap<TypeId, Box<dyn Resource>>,
    next_entity: Entity,
    root: Entity,
}

impl World {
    pub fn new() -> Self {
        let mut world = Self {
            components: HashMap::new(),
            next_entity: NULL_ENTITY + 1,
            root: NULL_ENTITY + 1,
        };

        world.register::<Transform>();
        world.register::<SceneNode>();

        world
            .builder()
            .with(Transform::default())
            .with(SceneNode::default())
            .build();

        world
    }

    pub fn root(&self) -> Entity {
        self.root
    }

    pub fn register<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();
        let storage = T::Storage::default();
        self.components.insert(type_id, Box::new(storage));
    }

    pub fn builder(&mut self) -> Builder {
        let new_entity = self.next_entity;
        self.next_entity += 1;
        Builder {
            entity: new_entity,
            world: self,
        }
    }

    pub fn storage<T: Component>(&self) -> Option<&T::Storage> {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)
            .map(|r| unsafe { r.downcast_ref_unchecked::<T::Storage>() })
    }

    pub fn storage_mut<T: Component>(&mut self) -> Option<&mut T::Storage> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)
            .map(|r| unsafe { r.downcast_mut_unchecked::<T::Storage>() })
    }

    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        let storage = self.storage::<T>().expect(
            format!(
                "Attempting to get unregistered component {}",
                std::any::type_name::<T>()
            )
            .as_str(),
        );

        if storage.contains(entity) {
            Some(unsafe { storage.get_unchecked(entity) })
        } else {
            None
        }
    }

    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        let storage = self
            .storage_mut::<T>()
            .expect("Attempting to get unregistered component");

        if storage.contains(entity) {
            Some(unsafe { storage.get_unchecked_mut(entity) })
        } else {
            None
        }
    }

    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let storage = self
            .storage_mut::<T>()
            .expect("Attempting to get unregistered component");

        if storage.contains(entity) {
            unsafe { *storage.get_unchecked_mut(entity) = component };
        } else {
            unsafe { storage.insert_unchecked(entity, component) };
        }
    }
}

// ---------- Components ---------- //

#[derive(Clone, Copy)]
pub struct SceneNode {
    first: Entity,
    next: Entity,
    prev: Entity,
    parent: Entity,
}

impl Default for SceneNode {
    fn default() -> Self {
        Self {
            first: NULL_ENTITY,
            next: NULL_ENTITY,
            prev: NULL_ENTITY,
            parent: NULL_ENTITY,
        }
    }
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

#[derive(Default, Clone, Copy)]
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

#[derive(Default, Clone)]
pub struct Light {
    pub dir: glam::Vec3,
    pub framebuffer_name: String,
}

impl Light {
    pub fn new_directional(dir: glam::Vec3, bounds: Bounds) -> Self {
        Self {
            dir,
            framebuffer_name: "shadow_out1".to_string(),
        }
    }

    pub fn calc_view_proj(&self, bounds: &Bounds) -> (glam::Mat4, glam::Mat4) {
        let scene_center = (bounds.0 + bounds.1) * 0.5;
        let scene_radius = (bounds.1 - scene_center).length();

        let dist_padding = 0.0;

        let light_pos = scene_center - self.dir * (scene_radius + dist_padding);
        let view = glam::Mat4::look_at_rh(light_pos, scene_center, glam::Vec3::Y);
        let proj = glam::Mat4::orthographic_rh(
            -scene_radius,
            scene_radius,
            -scene_radius,
            scene_radius,
            0.01,
            scene_radius * 2.0,
        );

        (view, proj)
    }
}

// ---------- DFS variants ---------- //

impl World {
    pub fn dfs<F>(&self, root: Entity, mut func: F)
    where
        F: FnMut(Entity, &World),
    {
        let mut stack = vec![root];
        while !stack.is_empty() {
            let cur = stack.pop().unwrap();
            func(cur, self);

            let mut child_itr = self
                .get::<SceneNode>(cur)
                .unwrap_or(&SceneNode::default())
                .first;
            while child_itr != NULL_ENTITY {
                stack.push(child_itr);
                child_itr = self
                    .get::<SceneNode>(child_itr)
                    .unwrap_or(&SceneNode::default())
                    .next;
            }
        }
    }

    pub fn dfs_mut<F>(&mut self, root: Entity, mut func: F)
    where
        F: FnMut(Entity, &mut World),
    {
        let mut stack = vec![root];
        while !stack.is_empty() {
            let cur = stack.pop().unwrap();
            func(cur, self);

            let mut child_itr = self
                .get::<SceneNode>(cur)
                .unwrap_or(&SceneNode::default())
                .first;
            while child_itr != NULL_ENTITY {
                stack.push(child_itr);
                child_itr = self
                    .get::<SceneNode>(child_itr)
                    .unwrap_or(&SceneNode::default())
                    .next;
            }
        }
    }

    pub fn dfs_acc<'a, T: Component + Clone, F>(&self, root: Entity, acc_init: T, mut func: F)
    where
        F: FnMut(Entity, &T) -> T,
    {
        let mut entity_stack = vec![root];
        let mut acc_stack = vec![acc_init.clone()];
        while !entity_stack.is_empty() {
            let cur_entity = entity_stack.pop().unwrap();
            let cur_acc = acc_stack.pop().unwrap();

            let mut child_itr = self
                .get::<SceneNode>(cur_entity)
                .unwrap_or(&SceneNode::default())
                .first;
            while child_itr != NULL_ENTITY {
                let acc = func(child_itr, &cur_acc);

                acc_stack.push(acc);
                entity_stack.push(child_itr);

                child_itr = self
                    .get::<SceneNode>(child_itr)
                    .unwrap_or(&SceneNode::default())
                    .next;
            }
        }
    }
}

// ---------- Bounds Calculations ---------- //

impl World {
    pub fn calc_bounds(&self, root: Entity) -> Bounds {
        let mut bounds = new_bounds();
        self.dfs(root, |e, w| {
            let cur_bounds = w
                .get::<Bounds>(e)
                .unwrap_or(&(glam::Vec3::ZERO, glam::Vec3::ZERO));
            let cur_transform = w
                .get::<Transform>(e)
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

/*
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

pub struct EntityID {
    pub id: u64,
}

impl Component for EntityID {
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
    pub dir: glam::Vec3,
    pub framebuffer_name: String,
}

impl Light {
    pub fn new_directional(dir: glam::Vec3, bounds: Bounds) -> Self {
        Self {
            dir,
            framebuffer_name: "shadow_out1".to_string(),
        }
    }

    pub fn calc_view_proj(&self, bounds: &Bounds) -> (glam::Mat4, glam::Mat4) {
        let scene_center = (bounds.0 + bounds.1) * 0.5;
        let scene_radius = (bounds.1 - scene_center).length();

        let dist_padding = 0.0;

        let light_pos = scene_center - self.dir * (scene_radius + dist_padding);
        let view = glam::Mat4::look_at_rh(light_pos, scene_center, glam::Vec3::Y);
        let proj = glam::Mat4::orthographic_rh(
            -scene_radius,
            scene_radius,
            -scene_radius,
            scene_radius,
            0.01,
            scene_radius * 2.0,
        );

        (view, proj)
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
*/
