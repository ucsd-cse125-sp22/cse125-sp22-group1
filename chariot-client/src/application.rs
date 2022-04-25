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
        renderer.register_pass(
            "boring",
            &direct_graphics_depth_pass!(include_str!("shader.wgsl"), wgpu::IndexFormat::Uint16),
        );

        renderer.register_pass(
            "forward",
            &indirect_graphics_depth_pass!(
                include_str!("shader.wgsl"),
                wgpu::IndexFormat::Uint16,
                [wgpu::TextureFormat::Rgba16Float]
            ),
        );

        renderer.register_pass(
            "postprocess",
            &direct_graphics_nodepth_pass!(
                include_str!("postprocess.wgsl"),
                wgpu::IndexFormat::Uint16
            ),
        );

        let (depth_tex, color_tex, fb_desc) =
            depth_color_framebuffer(&renderer, wgpu::TextureFormat::Rgba16Float);
        renderer.register_framebuffer("forward_out", fb_desc, [depth_tex, color_tex]);

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
        let model = glam::Mat4::from_scale(glam::vec3(0.3, 0.3, 0.3))
            * glam::Mat4::from_rotation_y(f32::to_radians(180.0))
            * glam::Mat4::from_rotation_x(f32::to_radians(90.0));

        let mut render_job = render_job::RenderJob::default();
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
