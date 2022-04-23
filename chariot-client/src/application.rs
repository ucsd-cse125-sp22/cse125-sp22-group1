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

pub struct Application {
    pub drawables: Vec<StaticMeshDrawable>,
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

        Self {
            drawables: import_result.unwrap().drawables,
            renderer: renderer,
            resources: resources,
        }
    }

    pub fn render(&mut self) {
        let view =
            glam::Mat4::look_at_rh(glam::vec3(0.0, 0.0, -2.0), glam::Vec3::ZERO, glam::Vec3::Y);
        let proj = glam::Mat4::perspective_rh(f32::to_radians(60.0), 1.0, 0.1, 100.0);
        let proj_view = proj * view;
        let model = glam::Mat4::IDENTITY;

        let mut render_job = render_job::RenderJob::new();
        for drawable in self.drawables.iter() {
            drawable.update_xforms(&self.renderer, &proj_view, &model);
            let render_graph = drawable.render_graph(&self.resources);
            render_job.merge_graph(render_graph);
        }

        self.renderer.render(&render_job);
    }

    pub fn update(&mut self) {
        //self.world.root_mut().update();
    }

    // TODO: input handlers
}
