use crate::drawable::*;
use crate::resources::{accum_bounds, new_bounds, Bounds};
use std::any::{Any, TypeId};
use std::borrow::Borrow;
use std::boxed::Box;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

pub mod components;
use components::*;

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
