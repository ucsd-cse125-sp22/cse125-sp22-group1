use gltf::Texture;
use specs::{Join, WorldExt};
use std::{
    cmp::Eq,
    collections::HashMap,
    default,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::drawable::*;
use crate::renderer::*;
use crate::resources::*;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: glam::Mat4 = glam::const_mat4!(
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 0.5, 0.5],
    [0.0, 0.0, 0.0, 1.0]
);

pub struct Application {
    pub drawables: Vec<StaticMeshDrawable>,
    pub renderer: Renderer,
    pub resources: ResourceManager,
    rot: f32,
}

impl Application {
    pub fn new(renderer: Renderer) -> Self {
        Self {
            drawables: Vec::new(),
            renderer: renderer,
            resources: ResourceManager::new(),
            rot: 0.0,
        }
    }

    pub fn render(&mut self) {
        let view =
            glam::Mat4::look_at_rh(glam::vec3(0.0, 0.0, -2.0), glam::Vec3::ZERO, glam::Vec3::Y);
        let proj = glam::Mat4::perspective_rh(f32::to_radians(60.0), 1.0, 0.1, 100.0);
        let proj_view = proj * view;
        let model = glam::Mat4::from_translation(glam::vec3(0.0, 0.0, 0.0))
            * glam::Mat4::from_axis_angle(glam::Vec3::Y, f32::to_radians(self.rot))
            * glam::Mat4::from_axis_angle(glam::Vec3::X, f32::to_radians(90.0))
            * glam::Mat4::from_scale(glam::vec3(0.3, 0.3, 0.3));

        self.rot += 1.0;

        let mut render_job = render_job::RenderJob::new();
        for drawable in self.drawables.iter() {
            drawable.update_xforms(&self.renderer, &proj_view, &model);
            let render_item = drawable.render_item(&self.resources);
            render_job.add_item(render_item);
        }

        self.renderer.render(&render_job);
    }

    pub fn update(&mut self) {}

    // TODO: input handlers
}
