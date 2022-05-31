use wgpu::util::DeviceExt;

use crate::renderer::Renderer;

use super::Bounds;

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
    pub surfel_points_buf: Option<wgpu::Buffer>,
    pub surfel_normals_buf: Option<wgpu::Buffer>,
    pub surfel_colors_buf: Option<wgpu::Buffer>,
    pub num_surfels: u32,
}

impl StaticMesh {
    pub fn vertex_buffer_slices(&self, submesh_idx: usize) -> Vec<wgpu::BufferSlice> {
        self.vertex_buffers
            .iter()
            .zip(self.submeshes[submesh_idx].vertex_ranges.iter())
            .map(|(buffer, range)| buffer.slice(*range))
            .collect::<Vec<wgpu::BufferSlice>>()
    }

    pub fn index_buffer_slice(&self, submesh_idx: usize) -> Option<wgpu::BufferSlice> {
        match &self.index_buffer {
            Some(buffer) => Some(buffer.slice(self.submeshes[submesh_idx].index_range.unwrap())),
            None => None,
        }
    }

    pub fn num_elements(&self, submesh_idx: usize) -> u32 {
        self.submeshes[submesh_idx].num_elements
    }
}

// Helper struct for building meshes
pub struct MeshBuilder<'a> {
    renderer: &'a Renderer,
    label: wgpu::Label<'a>,
    pub vertex_buffers: Vec<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    index_format: wgpu::IndexFormat,
    submeshes: Vec<SubMesh>,
    surfel_points_buf: Option<wgpu::Buffer>,
    surfel_normals_buf: Option<wgpu::Buffer>,
    surfel_colors_buf: Option<wgpu::Buffer>,
    num_surfels: u32,
}

impl<'a> MeshBuilder<'a> {
    pub fn new(renderer: &'a Renderer, name: Option<&'a str>) -> Self {
        Self {
            renderer: renderer,
            label: name,
            vertex_buffers: vec![],
            index_buffer: None,
            index_format: wgpu::IndexFormat::Uint16,
            submeshes: vec![],
            surfel_points_buf: None,
            surfel_normals_buf: None,
            surfel_colors_buf: None,
            num_surfels: 0,
        }
    }

    pub fn vertex_buffer<T: bytemuck::Pod>(&mut self, data: &[T]) -> &mut Self {
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

    pub fn _vertex_buffer_raw(&mut self, data: &[u8], _stride: usize) -> &mut Self {
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

    pub fn index_buffer<T: bytemuck::Pod>(
        &mut self,
        data: &[T],
        format: wgpu::IndexFormat,
    ) -> &mut Self {
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

    pub fn _index_buffer_raw(
        &mut self,
        data: &[u8],
        _stride: usize,
        format: wgpu::IndexFormat,
    ) -> &mut Self {
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

    pub fn _submesh(&mut self, vertex_ranges: &[IndexRange], num_elements: u32) -> &mut Self {
        self.submeshes.push(SubMesh {
            vertex_ranges: Vec::from(vertex_ranges),
            index_range: None,
            num_elements: num_elements,
        });
        self
    }

    pub fn indexed_submesh(
        &mut self,
        vertex_ranges: &[IndexRange],
        index_range: IndexRange,
        num_elements: u32,
    ) -> &mut Self {
        self.submeshes.push(SubMesh {
            vertex_ranges: Vec::from(vertex_ranges),
            index_range: Some(index_range),
            num_elements: num_elements,
        });
        self
    }

    pub fn surfels<'b>(
        &'b mut self,
        points: &[glam::Vec3],
        normals: &[glam::Vec3],
        colors: &[glam::Vec3],
    ) -> &'b mut Self {
        self.surfel_points_buf = Some(self.renderer.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: self.label,
                contents: unsafe {
                    core::slice::from_raw_parts(
                        points.as_ptr() as *const u8,
                        std::mem::size_of::<glam::Vec3>() * points.len(),
                    )
                },
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));
        self.surfel_normals_buf = Some(self.renderer.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: self.label,
                contents: unsafe {
                    core::slice::from_raw_parts(
                        normals.as_ptr() as *const u8,
                        std::mem::size_of::<glam::Vec3>() * normals.len(),
                    )
                },
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));
        self.surfel_colors_buf = Some(self.renderer.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: self.label,
                contents: unsafe {
                    core::slice::from_raw_parts(
                        colors.as_ptr() as *const u8,
                        std::mem::size_of::<glam::Vec3>() * colors.len(),
                    )
                },
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));
        self.num_surfels = points.len() as u32;
        self
    }

    pub fn produce_static_mesh(&mut self) -> StaticMesh {
        StaticMesh {
            vertex_buffers: std::mem::take(&mut self.vertex_buffers),
            index_buffer: std::mem::take(&mut self.index_buffer),
            index_format: self.index_format,
            submeshes: std::mem::take(&mut self.submeshes),
            surfel_points_buf: std::mem::take(&mut self.surfel_points_buf),
            surfel_normals_buf: std::mem::take(&mut self.surfel_normals_buf),
            surfel_colors_buf: std::mem::take(&mut self.surfel_colors_buf),
            num_surfels: self.num_surfels,
        }
    }
}
