use chariot_core::GLOBAL_CONFIG;

use super::Drawable;
use super::RenderContext;
use crate::drawable::util::TransformUniform;
use crate::renderer::render_job;
use crate::renderer::util;
use crate::renderer::Renderer;
use crate::resources::{MaterialHandle, ResourceManager, StaticMeshHandle};

// TODO: merge this with the forward pass and make this write depth. I got it to work but it was weird since on certain frames
// the particles wouldn't show up and the outline effect was being applied to the quads

mod particle_drawable {
    use crate::drawable::util::TransformUniform;
    use once_cell::sync::OnceCell;

    pub static VIEW_PROJ: OnceCell<TransformUniform<1>> = OnceCell::new();
}

pub struct ParticleDrawable {
    static_mesh_handle: StaticMeshHandle,
    material_handle: MaterialHandle,
    model_xform: TransformUniform<1>,
}

impl ParticleDrawable {
    pub const PASS_NAME: &'static str = "particle";
    const FRAMEBUFFER_NAME: &'static str = "geometry_out";
    pub fn new(
        renderer: &Renderer,
        static_mesh_handle: StaticMeshHandle,
        material_handle: MaterialHandle,
    ) -> Self {
        Self {
            static_mesh_handle,
            material_handle,
            model_xform: TransformUniform::new(renderer, Self::PASS_NAME, 1),
        }
    }
}

impl ParticleDrawable {
    pub fn update_model(&self, renderer: &Renderer, model: glam::Mat4) {
        self.model_xform.update(renderer, &[model]);
    }
}

impl Drawable for ParticleDrawable {
    fn register(renderer: &mut Renderer) {
        renderer.register_pass(
            Self::PASS_NAME,
            &util::indirect_graphics_depth_pass!(
                GLOBAL_CONFIG.get_shader_file_path("particle.wgsl"),
                false,
                [
                    wgpu::TextureFormat::Rgba16Float,
                    wgpu::TextureFormat::Rgba8Unorm
                ],
                [
                    Some(wgpu::BlendState::ALPHA_BLENDING),
                    Some(wgpu::BlendState::ALPHA_BLENDING)
                ]
            ),
        );

        particle_drawable::VIEW_PROJ.set(TransformUniform::new(renderer, Self::PASS_NAME, 0));
    }

    fn update_once(renderer: &Renderer, context: &RenderContext) {
        let view_ufm = particle_drawable::VIEW_PROJ.get().unwrap();

        let view_proj = context.proj * context.view;
        view_ufm.update(renderer, &[view_proj]);
    }

    fn render_graph<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderGraph<'a> {
        let static_mesh = context
            .resources
            .meshes
            .get(&self.static_mesh_handle)
            .expect("invalid static mesh handle");
        let material = context
            .resources
            .materials
            .get(&self.material_handle)
            .expect("invalid material handle");

        let view_bind_group = &particle_drawable::VIEW_PROJ.get().unwrap().bind_group;
        let model_bind_group = &self.model_xform.bind_group;
        let mut bind_group_refs = vec![view_bind_group, model_bind_group];
        bind_group_refs.extend(material.bind_groups(context.iteration));
        render_job::RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: context.framebuffer_name(Self::FRAMEBUFFER_NAME),
            num_elements: static_mesh.num_elements(0),
            vertex_buffers: static_mesh.vertex_buffer_slices(0),
            index_buffer: static_mesh.index_buffer_slice(0),
            index_format: static_mesh.index_format,
            bind_group: bind_group_refs,
        }
        .to_graph()
    }
}
