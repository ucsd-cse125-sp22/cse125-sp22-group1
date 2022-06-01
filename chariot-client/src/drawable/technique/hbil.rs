use super::RenderContext;
use super::Technique;
use crate::assets::shaders;
use crate::drawable::util::TransformUniform;
use crate::renderer::util;

use crate::renderer::render_job::RenderItem;
use crate::renderer::Renderer;
use crate::resources::material::Material;
use crate::resources::material::MaterialBuilder;
use crate::resources::ResourceManager;
use crate::resources::StaticMeshHandle;

mod hbil_technique {
    use crate::drawable::util::TransformUniform;
    use once_cell::sync::OnceCell;

    pub static INV_VIEW_PROJ: OnceCell<TransformUniform<2>> = OnceCell::new();
}

pub struct HBILTechnique {
    quad_handle: StaticMeshHandle,
    material: Material,
}

impl HBILTechnique {
    const FRAMEBUFFER_NAME: &'static str = "hbil_out";
    pub fn new(
        renderer: &Renderer,
        resources: &ResourceManager,
        quad_handle: StaticMeshHandle,
    ) -> Self {
        let material = MaterialBuilder::new(renderer, resources, Self::PASS_NAME)
            .framebuffer_texture_resource(0, 0, "shade_direct_out", 0, false)
            .framebuffer_texture_resource(0, 1, "geometry_out", 1, false)
            .framebuffer_texture_resource(0, 2, "geometry_out", 2, false)
            .produce();

        Self {
            quad_handle,
            material,
        }
    }
}

impl Technique for HBILTechnique {
    const PASS_NAME: &'static str = "hbil";

    fn register(renderer: &mut Renderer) {
        renderer.register_pass(
            Self::PASS_NAME,
            &util::indirect_graphics_nodepth_pass!(
                &shaders::HBIL,
                false,
                [wgpu::TextureFormat::Rgba8Unorm],
                [Some(wgpu::BlendState::REPLACE)]
            ),
        );

        let res =
            hbil_technique::INV_VIEW_PROJ.set(TransformUniform::new(renderer, Self::PASS_NAME, 1));

        if res.is_err() {
            panic!("Can't register this technique twice!");
        }
    }

    fn update_once(renderer: &Renderer, context: &RenderContext) {
        let view_ufm = hbil_technique::INV_VIEW_PROJ.get().unwrap();

        let inv_view = context.view.inverse();
        let inv_proj = context.proj.inverse();
        view_ufm.update(renderer, &[inv_view, inv_proj]);
    }

    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> RenderItem<'a> {
        let static_mesh = context.resources.meshes.get(&self.quad_handle).unwrap();
        let view_bind_group = &hbil_technique::INV_VIEW_PROJ.get().unwrap().bind_group;

        let bind_groups = self
            .material
            .bind_groups(context.iteration)
            .into_iter()
            .chain(std::slice::from_ref(view_bind_group).iter())
            .collect();

        RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: context.framebuffer_name(Self::FRAMEBUFFER_NAME),
            num_elements: static_mesh.num_elements(0),
            vertex_buffers: static_mesh.vertex_buffer_slices(0),
            index_buffer: static_mesh.index_buffer_slice(0),
            index_format: wgpu::IndexFormat::Uint16,
            bind_group: bind_groups,
        }
    }
}

pub struct HBILDebayerTechnique {
    quad_handle: StaticMeshHandle,
    material: Material,
}

impl HBILDebayerTechnique {
    const FRAMEBUFFER_NAME: &'static str = "hbil_debayer_out";
    pub fn new(
        renderer: &Renderer,
        resources: &ResourceManager,
        quad_handle: StaticMeshHandle,
    ) -> Self {
        let material = MaterialBuilder::new(renderer, resources, Self::PASS_NAME)
            .framebuffer_texture_resource(0, 0, "hbil_out", 0, false)
            .produce();

        Self {
            quad_handle,
            material,
        }
    }
}

impl Technique for HBILDebayerTechnique {
    const PASS_NAME: &'static str = "hbil_debayer";

    fn register(renderer: &mut Renderer) {
        renderer.register_pass(
            Self::PASS_NAME,
            &util::indirect_graphics_nodepth_pass!(
                &shaders::HBIL_DEBAYER,
                false,
                [wgpu::TextureFormat::Rgba8Unorm],
                [Some(wgpu::BlendState::REPLACE)]
            ),
        );
    }

    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> RenderItem<'a> {
        let static_mesh = context.resources.meshes.get(&self.quad_handle).unwrap();
        let bind_groups = self.material.bind_groups(context.iteration);

        RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: context.framebuffer_name(Self::FRAMEBUFFER_NAME),
            num_elements: static_mesh.num_elements(0),
            vertex_buffers: static_mesh.vertex_buffer_slices(0),
            index_buffer: static_mesh.index_buffer_slice(0),
            index_format: wgpu::IndexFormat::Uint16,
            bind_group: bind_groups,
        }
    }
}
