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

pub struct UILayerTechnique {
    pub vertex_buffer: wgpu::Buffer,
    texcoord_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    material: Material,
}

impl UILayerTechnique {
    const FRAMEBUFFER_NAME: &'static str = "surface_nodepth";

    pub fn create_verts_data(pos: glam::Vec2, size: glam::Vec2) -> [[f32; 2]; 4] {
        let pos_ndc = glam::vec2(pos.x, 1.0 - pos.y) * 2.0 - 1.0;
        let size_ndc = glam::vec2(size.x, -size.y) * 2.0;
        [
            [pos_ndc.x, pos_ndc.y],
            [pos_ndc.x + size_ndc.x, pos_ndc.y],
            [pos_ndc.x + size_ndc.x, pos_ndc.y + size_ndc.y],
            [pos_ndc.x, pos_ndc.y + size_ndc.y],
        ]
    }

    pub fn new(
        renderer: &Renderer,
        pos: glam::Vec2,
        size: glam::Vec2,
        tc_pos: glam::Vec2,
        tc_size: glam::Vec2,
        texture: &wgpu::Texture,
    ) -> Self {
        let raw_verts_data = UILayerTechnique::create_verts_data(pos, size);
        let verts_data: &[u8] = bytemuck::cast_slice(&raw_verts_data);
        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ui_verts"),
                contents: bytemuck::cast_slice(&verts_data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
        let texcoord_data: [[f32; 2]; 4] = [
            [tc_pos.x, tc_pos.y],
            [tc_pos.x + tc_size.x, tc_pos.y],
            [tc_pos.x + tc_size.x, tc_pos.y + tc_size.y],
            [tc_pos.x, tc_pos.y + tc_size.y],
        ];
        let inds_data: [u16; 6] = [0, 2, 1, 0, 3, 2];

        let texcoord_buffer =
            renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("ui_texcoords"),
                    contents: bytemuck::cast_slice(&texcoord_data),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ui_inds"),
                contents: bytemuck::cast_slice(&inds_data),
                usage: wgpu::BufferUsages::INDEX,
            });

        let material = MaterialBuilder::new_no_fb(renderer, Self::PASS_NAME)
            .texture_resource(
                0,
                0,
                texture.create_view(&wgpu::TextureViewDescriptor::default()),
            )
            .produce();

        Self {
            vertex_buffer,
            texcoord_buffer,
            index_buffer,
            material,
        }
    }
}

impl Technique for UILayerTechnique {
    const PASS_NAME: &'static str = "ui";

    fn register(renderer: &mut Renderer) {
        renderer.register_pass(
            Self::PASS_NAME,
            &util::direct_graphics_nodepth_pass!(GLOBAL_CONFIG.get_shader_file_path("ui.wgsl")),
        );
    }

    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> RenderItem<'a> {
        let bind_groups = self.material.bind_groups(context.iteration);

        RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: context.framebuffer_name(Self::FRAMEBUFFER_NAME),
            num_elements: 6,
            vertex_buffers: vec![self.vertex_buffer.slice(..), self.texcoord_buffer.slice(..)],
            index_buffer: Some(self.index_buffer.slice(..)),
            index_format: wgpu::IndexFormat::Uint16,
            bind_group: bind_groups,
        }
    }
}
