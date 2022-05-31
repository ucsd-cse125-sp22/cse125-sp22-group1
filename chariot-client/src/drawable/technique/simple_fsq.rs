use super::RenderContext;
use super::Technique;
use crate::drawable::util::TransformUniform;
use crate::renderer::util;

use chariot_core::GLOBAL_CONFIG;
use wgpu::util::DeviceExt;

use crate::renderer::render_job::RenderItem;
use crate::renderer::Renderer;
use crate::resources::material::Material;
use crate::resources::material::MaterialBuilder;
use crate::resources::ResourceManager;
use crate::resources::StaticMeshHandle;

pub struct SimpleFSQTechnique {
    quad_handle: StaticMeshHandle,
    material: Material,
}

impl SimpleFSQTechnique {
    const FRAMEBUFFER_NAME: &'static str = "surface_nodepth";
    pub fn new(
        renderer: &Renderer,
        resources: &ResourceManager,
        framebuffer_name: &str,
        framebuffer_idx: usize,
        quad_handle: StaticMeshHandle,
    ) -> Self {
        let material = MaterialBuilder::new(renderer, resources, Self::PASS_NAME)
            .framebuffer_texture_resource(0, 0, framebuffer_name, framebuffer_idx, false)
            .produce();

        Self {
            quad_handle,
            material,
        }
    }
}

impl Technique for SimpleFSQTechnique {
    const PASS_NAME: &'static str = "simple_fsq";

    fn register(renderer: &mut Renderer) {
        renderer.register_pass(
            Self::PASS_NAME,
            &util::direct_graphics_nodepth_pass!(
                GLOBAL_CONFIG.get_shader_file_path("simple_fsq.wgsl")
            ),
        );
    }

    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> RenderItem<'a> {
        let static_mesh = context.resources.meshes.get(&self.quad_handle).unwrap();
        let bind_groups = self.material.bind_groups(context.iteration);

        RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: Self::FRAMEBUFFER_NAME.to_string(),
            num_elements: static_mesh.num_elements(0),
            vertex_buffers: static_mesh.vertex_buffer_slices(0),
            index_buffer: static_mesh.index_buffer_slice(0),
            index_format: wgpu::IndexFormat::Uint16,
            bind_group: bind_groups,
        }
    }
}
