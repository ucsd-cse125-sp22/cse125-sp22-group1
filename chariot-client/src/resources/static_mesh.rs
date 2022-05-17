use wgpu::util::DeviceExt;

use crate::renderer::Renderer;

pub type IndexRange = (std::ops::Bound<u64>, std::ops::Bound<u64>);

/*
 * Theoretically, a single mesh could have multiple draw calls. For example, a car mesh could
 * have one mesh for the body that uses a "shiny car body" material, another mesh for the windows
 * which use another "glass window" material, and another for the tires which have their own material.
 * However, all these draw calls use different slices of the same index and vertex buffers.
 *
 * To simplify things, for now I require submeshes to all have the same material. During gltf import I just
 * make a new static mesh for each gltf primitive (submesh equivalent).
 */
pub struct SubMesh {
    pub vertex_ranges: Vec<IndexRange>,
    pub index_range: Option<IndexRange>,
    pub num_elements: u32,
}

pub struct StaticMesh {
    pub vertex_buffers: Vec<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
    pub index_format: wgpu::IndexFormat,
    pub submeshes: Vec<SubMesh>,
}

// Helper struct for building meshes
pub struct MeshBuilder<'a> {
    renderer: &'a Renderer,
    label: wgpu::Label<'a>,
    pub vertex_buffers: Vec<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    index_format: wgpu::IndexFormat,
    submeshes: Vec<SubMesh>,
}

impl<'a> MeshBuilder<'a> {
    pub fn new(renderer: &'a Renderer, name: Option<&'a str>) -> Self {
        MeshBuilder {
            renderer: renderer,
            label: name,
            vertex_buffers: Vec::new(),
            index_buffer: None,
            index_format: wgpu::IndexFormat::Uint16,
            submeshes: Vec::new(),
        }
    }

    pub fn vertex_buffer<'b, T: bytemuck::Pod>(&'b mut self, data: &[T]) -> &'b mut Self {
        let vertex_buffer =
            self.renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: self.label,
                    contents: bytemuck::cast_slice(data),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        self.vertex_buffers.push(vertex_buffer);
        self
    }

    pub fn _vertex_buffer_raw<'b>(&'b mut self, data: &[u8], _stride: usize) -> &'b mut Self {
        let vertex_buffer =
            self.renderer
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: self.label,
                    contents: data,
                    usage: wgpu::BufferUsages::VERTEX,
                });
        self.vertex_buffers.push(vertex_buffer);
        self
    }

    pub fn index_buffer<'b, T: bytemuck::Pod>(
        &'b mut self,
        data: &[T],
        format: wgpu::IndexFormat,
    ) -> &'b mut Self {
        self.index_format = format;
        self.index_buffer = Some(self.renderer.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: self.label,
                contents: bytemuck::cast_slice(data),
                usage: wgpu::BufferUsages::INDEX,
            },
        ));
        self
    }

    pub fn _index_buffer_raw<'b>(
        &'b mut self,
        data: &[u8],
        _stride: usize,
        format: wgpu::IndexFormat,
    ) -> &'b mut Self {
        self.index_format = format;
        self.index_buffer = Some(self.renderer.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: self.label,
                contents: data,
                usage: wgpu::BufferUsages::INDEX,
            },
        ));
        self
    }

    pub fn _submesh<'b>(
        &'b mut self,
        vertex_ranges: &[IndexRange],
        num_elements: u32,
    ) -> &'b mut Self {
        self.submeshes.push(SubMesh {
            vertex_ranges: Vec::from(vertex_ranges),
            index_range: None,
            num_elements: num_elements,
        });
        self
    }

    pub fn indexed_submesh<'b>(
        &'b mut self,
        vertex_ranges: &[IndexRange],
        index_range: IndexRange,
        num_elements: u32,
    ) -> &'b mut Self {
        self.submeshes.push(SubMesh {
            vertex_ranges: Vec::from(vertex_ranges),
            index_range: Some(index_range),
            num_elements: num_elements,
        });
        self
    }

    pub fn produce_static_mesh(&mut self) -> StaticMesh {
        StaticMesh {
            vertex_buffers: std::mem::take(&mut self.vertex_buffers),
            index_buffer: std::mem::take(&mut self.index_buffer),
            index_format: self.index_format,
            submeshes: std::mem::take(&mut self.submeshes),
        }
    }
}
