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

// bloom from https://www.froyok.fr/blog/2021-09-ue4-custom-lens-flare/

pub struct DownsampleBloomTechnique {
    quad_handle: StaticMeshHandle,
    material: Material,
}

impl DownsampleBloomTechnique {
    const FRAMEBUFFER_NAME: &'static str = "downsample_bloom_out";
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
            .framebuffer_texture_resource(0, 0, "shade_direct_out_0_ds", 0, false)
            .sampler_resource(0, 1, sampler)
            .produce();

        Self {
            quad_handle,
            material,
        }
    }
}

impl Technique for DownsampleBloomTechnique {
    const PASS_NAME: &'static str = "downsample_bloom";

    fn register(renderer: &mut Renderer) {
        renderer.register_pass(
            Self::PASS_NAME,
            &util::indirect_graphics_nodepth_pass!(
                GLOBAL_CONFIG.get_shader_file_path("downsample_bloom.wgsl"),
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

pub struct KawaseBlurDownTechnique {
    quad_handle: StaticMeshHandle,
    material: Material,
}

impl KawaseBlurDownTechnique {
    const FRAMEBUFFER_NAME: &'static str = "kawase_blur_down_out";
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
            .framebuffer_texture_resource(0, 0, "downsample_bloom_out", 0, false)
            .sampler_resource(0, 1, sampler)
            .produce();

        Self {
            quad_handle,
            material,
        }
    }
}

impl Technique for KawaseBlurDownTechnique {
    const PASS_NAME: &'static str = "kawase_blur_down";
    fn register(renderer: &mut Renderer) {
        renderer.register_pass(
            Self::PASS_NAME,
            &util::indirect_graphics_nodepth_pass!(
                GLOBAL_CONFIG.get_shader_file_path("kawase_blur_down.wgsl"),
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

pub struct KawaseBlurUpTechnique {
    quad_handle: StaticMeshHandle,
    material: Material,
}

impl KawaseBlurUpTechnique {
    const FRAMEBUFFER_NAME: &'static str = "kawase_blur_up_out";
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
            .framebuffer_texture_resource(0, 0, "kawase_blur_down_out", 0, false)
            .sampler_resource(0, 1, sampler)
            .produce();

        Self {
            quad_handle,
            material,
        }
    }
}

impl Technique for KawaseBlurUpTechnique {
    const PASS_NAME: &'static str = "kawase_blur_up";

    fn register(renderer: &mut Renderer) {
        renderer.register_pass(
            Self::PASS_NAME,
            &util::indirect_graphics_nodepth_pass!(
                GLOBAL_CONFIG.get_shader_file_path("kawase_blur_up.wgsl"),
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

pub struct CompositeBloomTechnique {
    quad_handle: StaticMeshHandle,
    material: Material,
}

impl CompositeBloomTechnique {
    const FRAMEBUFFER_NAME: &'static str = "composite_bloom_out";
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
            .framebuffer_texture_resource(0, 0, "shade_direct_out", 0, false)
            .framebuffer_texture_resource(0, 1, "kawase_blur_up_out", 0, false)
            .framebuffer_texture_resource(0, 2, "hibl_debayer_out", 0, false)
            .sampler_resource(0, 3, sampler)
            .produce();

        Self {
            quad_handle,
            material,
        }
    }
}

impl Technique for CompositeBloomTechnique {
    const PASS_NAME: &'static str = "composite_bloom";
    fn register(renderer: &mut Renderer) {
        renderer.register_pass(
            Self::PASS_NAME,
            &util::indirect_graphics_nodepth_pass!(
                GLOBAL_CONFIG.get_shader_file_path("composite_bloom.wgsl"),
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
