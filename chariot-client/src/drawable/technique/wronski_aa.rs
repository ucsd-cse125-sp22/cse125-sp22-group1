use super::RenderContext;
use super::Technique;
use crate::assets::shaders;
use crate::renderer::util;

use crate::renderer::render_job::RenderItem;
use crate::renderer::Renderer;
use crate::resources::material::Material;
use crate::resources::material::MaterialBuilder;
use crate::resources::ResourceManager;
use crate::resources::StaticMeshHandle;

pub struct WronskiAATechnique {
    quad_handle: StaticMeshHandle,
    material: Material,
}

impl WronskiAATechnique {
    const FRAMEBUFFER_NAME: &'static str = "shade_direct_out";

    pub fn new(
        renderer: &Renderer,
        resources: &ResourceManager,
        quad_handle: StaticMeshHandle,
    ) -> Self {
        let sampler = renderer.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let material = MaterialBuilder::new(renderer, resources, Self::PASS_NAME)
            .framebuffer_texture_resource(0, 0, "shade_direct_out_us", 0, false)
            .sampler_resource(0, 1, sampler)
            .produce();

        Self {
            quad_handle,
            material,
        }
    }
}

impl Technique for WronskiAATechnique {
    const PASS_NAME: &'static str = "wronski_aa";

    fn register(renderer: &mut Renderer) {
        renderer.register_pass(
            Self::PASS_NAME,
            &util::indirect_graphics_nodepth_pass!(
                &shaders::WRONSKI_AA,
                false,
                [wgpu::TextureFormat::Rgba16Float],
                [Some(wgpu::BlendState::ALPHA_BLENDING)]
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
