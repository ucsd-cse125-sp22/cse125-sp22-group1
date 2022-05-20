use std::ops::Bound;
use std::ops::RangeBounds;

pub mod geometry;
pub mod probe;
pub mod shade;
pub mod shadow;

pub use geometry::*;
pub use probe::*;
pub use shade::*;
pub use shadow::*;

use crate::drawable::*;
use crate::renderer::*;
use crate::resources::*;
use wgpu::util::DeviceExt;
use wgpu::BufferAddress;

pub trait Technique {
    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderItem<'a>;
}

pub struct TransformUniform<const NUM_ELEMS: usize> {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl<const NUM_ELEMS: usize> TransformUniform<NUM_ELEMS> {
    pub fn new(renderer: &Renderer, pass_name: &str, group: u32) -> Self {
        let uniform_init = [glam::Mat4::IDENTITY; NUM_ELEMS];
        let xform_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("uniform_buf"),
                contents: unsafe {
                    core::slice::from_raw_parts(
                        uniform_init.as_ptr() as *const u8,
                        std::mem::size_of::<[glam::Mat4; NUM_ELEMS]>(),
                    )
                },
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::MAP_WRITE,
            });

        let xform_bind_group =
            renderer.create_bind_group(pass_name, group, &[(0, xform_buffer.as_entire_binding())]);

        return TransformUniform {
            buffer: xform_buffer,
            bind_group: xform_bind_group,
        };
    }

    pub fn update(&self, renderer: &Renderer, data: &[glam::Mat4; NUM_ELEMS]) {
        renderer.write_buffer(&self.buffer, data);
    }
}
