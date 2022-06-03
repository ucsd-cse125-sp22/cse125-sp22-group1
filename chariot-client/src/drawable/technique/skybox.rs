use super::RenderContext;
use super::Technique;
use crate::assets::shaders;
use crate::drawable::util::TransformUniform;

use crate::renderer::render_job::RenderItem;
use crate::renderer::util;
use crate::renderer::Renderer;
use crate::resources::ResourceManager;
use crate::resources::StaticMeshHandle;

mod skybox_technique {
    use crate::drawable::util::TransformUniform;
    use once_cell::sync::OnceCell;

    pub static INV_VIEW_PROJ: OnceCell<TransformUniform<2>> = OnceCell::new();
}

pub struct SkyboxTechnique {
    quad_handle: StaticMeshHandle,
}

impl SkyboxTechnique {
    const FRAMEBUFFER_NAME: &'static str = "shade_direct_out";
    pub fn new(_: &Renderer, _: &ResourceManager, quad_handle: StaticMeshHandle) -> Self {
        Self { quad_handle }
    }
}

impl Technique for SkyboxTechnique {
    const PASS_NAME: &'static str = "skybox";

    fn register(renderer: &mut Renderer) {
        renderer.register_pass(
            Self::PASS_NAME,
            &util::indirect_graphics_nodepth_pass!(
                &shaders::SKYBOX,
                false,
                [wgpu::TextureFormat::Rgba16Float],
                [Some(wgpu::BlendState::ALPHA_BLENDING)]
            ),
        );

        let res = skybox_technique::INV_VIEW_PROJ.set(TransformUniform::new(
            renderer,
            Self::PASS_NAME,
            0,
        ));

        if res.is_err() {
            println!("Re-registering technique but not resetting static uniforms");
        }
    }

    fn update_once(renderer: &Renderer, context: &RenderContext) {
        let view_ufm = skybox_technique::INV_VIEW_PROJ.get().unwrap();

        let inv_view = context.view.inverse();
        let inv_proj = context.proj.inverse();
        view_ufm.update(renderer, &[inv_view, inv_proj]);
    }

    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> RenderItem<'a> {
        let static_mesh = context.resources.meshes.get(&self.quad_handle).unwrap();
        let view_bind_group = &skybox_technique::INV_VIEW_PROJ.get().unwrap().bind_group;
        RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: context.framebuffer_name(Self::FRAMEBUFFER_NAME),
            num_elements: static_mesh.num_elements(0),
            vertex_buffers: static_mesh.vertex_buffer_slices(0),
            index_buffer: static_mesh.index_buffer_slice(0),
            index_format: wgpu::IndexFormat::Uint16,
            bind_group: vec![view_bind_group],
        }
    }
}
