use gltf::Texture;
use std::{
    cmp::Eq,
    collections::HashMap,
    default,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::drawable::*;
use crate::renderer::*;
use crate::resources::*;
use crate::scenegraph::*;

pub struct Application {
    pub world: World,
    pub renderer: Renderer,
    pub resources: ResourceManager,
}

impl Application {
    pub fn new(mut renderer: Renderer) -> Self {
        let mut resources = ResourceManager::new();

        let import_result = resources.import_gltf(&mut renderer, "models/DamagedHelmet.glb");

        if !import_result.is_ok() {
            panic!("Failed to import model");
        }

        let mut world = World::new();
        let mut helmet = Entity::new();
        helmet.set_component(Transform {
            translation: glam::Vec3::ZERO,
            rotation: glam::Quat::from_axis_angle(glam::Vec3::X, f32::to_radians(90.0)),
            scale: glam::vec3(0.3, 0.3, 0.3),
        });

        helmet.set_component(import_result.unwrap().drawables);

        world.root_mut().add_child(helmet);

        Self {
            world: world,
            renderer: renderer,
            resources: resources,
        }
    }

    pub fn render(&mut self) {
        let view =
            glam::Mat4::look_at_rh(glam::vec3(0.0, 0.0, -2.0), glam::Vec3::ZERO, glam::Vec3::Y);
        let proj = glam::Mat4::perspective_rh(f32::to_radians(60.0), 1.0, 0.1, 100.0);
        let proj_view = proj * view;

        let mut render_job = render_job::RenderJob::new();
        let root_transform = self
            .world
            .root()
            .get_component::<Transform>()
            .unwrap_or(&Transform::default())
            .to_mat4();
        dfs_acc(self.world.root_mut(), root_transform, |e, acc| {
            let cur_model = e
                .get_component::<Transform>()
                .unwrap_or(&Transform::default())
                .to_mat4();
            let acc_model = *acc * cur_model;

            if let Some(drawables) = e.get_component::<Vec<StaticMeshDrawable>>() {
                for drawable in drawables.iter() {
                    drawable.update_xforms(&self.renderer, &proj_view, &acc_model);
                    let render_graph = drawable.render_graph(&self.resources);
                    render_job.merge_graph(render_graph);
                }
            }

            acc_model
        });

        self.renderer.render(&render_job);
    }

    pub fn update(&mut self) {
        //self.world.root_mut().update();
        dfs_mut(self.world.root_mut(), &|e| {
            if let Some(transform) = e.get_component::<Transform>() {
                let rot_inc = glam::Quat::from_axis_angle(glam::Vec3::Y, 0.01);
                let new_rot = rot_inc * transform.rotation;
                let new_transform = Transform {
                    rotation: new_rot,
                    ..*transform
                };
                e.set_component(new_transform);
            }
        });
    }

    // TODO: input handlers
}
