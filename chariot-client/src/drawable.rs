use core::num;
use std::{collections::{HashMap}};
use wgpu::util::DeviceExt;

use crate::renderer::*;
use crate::application::*;

pub trait Drawable {
	fn render_item<'a>(&'a self, resources : &'a ResourceManager) -> RenderItem<'a>;
}

#[derive(Default)]
pub struct Material {
	pub pass_name : String,
	pub bind_groups : HashMap<u32, wgpu::BindGroup>,

	buffers : Vec<wgpu::Buffer>,
	textures : Vec<wgpu::TextureView>,
	samplers : Vec<wgpu::Sampler>
}

pub type IndexRange = (std::ops::Bound<u64>, std::ops::Bound<u64>);

pub struct SubMesh {
	vertex_ranges : Vec<IndexRange>,
	index_range : Option<IndexRange>,
	num_elements : u32
}

pub struct StaticMesh {
	vertex_buffers : Vec<wgpu::Buffer>,
	index_buffer : Option<wgpu::Buffer>,
	index_format : wgpu::IndexFormat,
	submeshes : Vec<SubMesh>
}

pub struct StaticMeshDrawable {
	material : MaterialHandle,
	static_mesh : StaticMeshHandle,
	submesh_idx : usize,
	xform_buffer : wgpu::Buffer,
	xform_bind_group : wgpu::BindGroup
}

impl StaticMeshDrawable {
	fn new(renderer : &Renderer, resources : &ResourceManager, material : MaterialHandle, static_mesh : StaticMeshHandle, submesh_idx : usize) -> Self {
		let uniform_init = glam::Mat4::IDENTITY;
		let xform_buffer = renderer.device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("uniform_buf"),
				contents: unsafe { 
					core::slice::from_raw_parts(
						core::slice::from_ref(&uniform_init).as_ptr() as *const u8, 
						std::mem::size_of::<glam::Mat4>()
					) 
				},
				usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
			}
		);
		let pass_name = &resources.materials.get(&material)
			.expect("invalid material handle")
			.pass_name;
		let xform_bind_group = renderer.create_bind_group(pass_name.as_str(), 0, 
			&[xform_buffer.as_entire_binding()]
		);

		StaticMeshDrawable { 
			material: material, 
			static_mesh: static_mesh, 
			submesh_idx: submesh_idx,
			xform_buffer: xform_buffer, 
			xform_bind_group: xform_bind_group 
		}
	}
}

impl Drawable for StaticMeshDrawable {
	fn render_item<'a>(&'a self, resources : &'a ResourceManager) -> RenderItem<'a> {
		let static_mesh = resources.meshes.get(&self.static_mesh)
			.expect("invalid static mesh handle");
		let material = resources.materials.get(&self.material)
			.expect("invalid material handle");

		let vertex_buffers_with_ranges = 
			static_mesh.vertex_buffers.iter().zip(static_mesh.submeshes[self.submesh_idx].vertex_ranges.iter());

		let mut bind_group_refs = vec![&self.xform_bind_group];
		bind_group_refs.extend(material.bind_groups.values());
		RenderItem::Graphics { 
            pass_name: material.pass_name.as_str(), 
            framebuffer_name: "surface", 
            num_elements: static_mesh.submeshes[self.submesh_idx].num_elements, 
            vertex_buffers: vertex_buffers_with_ranges.map(
				|(buffer, range)| { buffer.slice(*range) }
			).collect::<Vec<wgpu::BufferSlice>>(), 
            index_buffer: match &static_mesh.index_buffer {
				Some(buffer) => Some(
					buffer.slice(static_mesh.submeshes[self.submesh_idx].index_range.unwrap())
				),
				None => None
			}, 
            index_format: static_mesh.index_format, 
            bind_group: bind_group_refs
        }
	}
}

impl<'a> specs::Component for StaticMeshDrawable {
    // This uses `VecStorage`, because all entities have a position.
    type Storage = specs::VecStorage<StaticMeshDrawable>;
}

enum MatResourceIdx {
	Buffer(usize),
	Texture(usize),
	Sampler(usize)
}
struct MaterialBuilder<'a> {
	pass_name : &'a str,
	renderer : &'a Renderer,
	bind_group_resources : HashMap<u32, HashMap<u32, MatResourceIdx>>,

	buffers : Vec<wgpu::Buffer>,
	textures : Vec<wgpu::TextureView>,
	samplers : Vec<wgpu::Sampler>
}

impl<'a> MaterialBuilder<'a> {
	fn new(renderer : &'a Renderer, pass_name : &'a str) -> Self {
		MaterialBuilder { 
			pass_name, 
			renderer,
			bind_group_resources: HashMap::new(),
			buffers: Vec::new(),
			textures: Vec::new(),
			samplers: Vec::new()
		}
	}

	fn buffer_resource<'b>(&'b mut self, group : u32, binding : u32, buffer : wgpu::Buffer) -> &'b mut Self {
		self.buffers.push(buffer);

		self.bind_group_resources.entry(group)
			.or_default()
			.insert(binding, MatResourceIdx::Buffer(self.buffers.len() - 1));
		self
	}

	fn texture_resource<'b>(&'b mut self, group : u32, binding : u32, texture : wgpu::TextureView) -> &'b mut Self {
		self.textures.push(texture);

		self.bind_group_resources.entry(group)
			.or_default()
			.insert(binding, MatResourceIdx::Texture(self.textures.len() - 1));
		self
	}

	fn sampler_resource<'b>(&'b mut self, group : u32, binding : u32, sampler : wgpu::Sampler) -> &'b mut Self {
		self.samplers.push(sampler);

		self.bind_group_resources.entry(group)
			.or_default()
			.insert(binding, MatResourceIdx::Sampler(self.samplers.len() - 1));
		self
	}

	fn produce(&mut self) -> Material {
		let lookup_binding_resource = |(_, resource_idx) : (&u32, &MatResourceIdx)| 	{
			match resource_idx {
				MatResourceIdx::Buffer(idx) => self.buffers[*idx].as_entire_binding(),
				MatResourceIdx::Texture(idx) => wgpu::BindingResource::TextureView(
					&self.textures[*idx]
				),
				MatResourceIdx::Sampler(idx) => wgpu::BindingResource::Sampler(
					&self.samplers[*idx]
				)
			}
		};

		let create_bind_group = |(group, resource_map) : (&u32, &HashMap<u32, MatResourceIdx>)| {
			let binding_resources = resource_map.iter().map(lookup_binding_resource).collect::<Vec<wgpu::BindingResource>>();
			(*group, self.renderer.create_bind_group(self.pass_name, *group, &binding_resources))
		};

		let bind_groups = self.bind_group_resources.iter().map(create_bind_group).collect::<HashMap<u32, wgpu::BindGroup>>();

		Material { 
			pass_name: String::from(self.pass_name), 
			bind_groups: bind_groups, 
			buffers: std::mem::take(&mut self.buffers), 
			textures: std::mem::take(&mut self.textures), 
			samplers: std::mem::take(&mut self.samplers)
		}
	}
}

pub struct MeshBuilder<'a> {
	renderer : &'a Renderer,
	label : wgpu::Label<'a>,
	pub vertex_buffers : Vec<wgpu::Buffer>,
	index_buffer : Option<wgpu::Buffer>,
	index_format : wgpu::IndexFormat,
	submeshes : Vec<SubMesh>
}

impl<'a> MeshBuilder<'a> {
	pub fn new(renderer : &'a Renderer, name : Option<&'a str>) -> Self {
		MeshBuilder { 
			renderer: renderer, 
			label: name,
			vertex_buffers: Vec::new(), 
			index_buffer: None,
			index_format: wgpu::IndexFormat::Uint16,
			submeshes: Vec::new()
		}
	}

	pub fn vertex_buffer<'b, T : bytemuck::Pod>(&'b mut self, data : &[T]) -> &'b mut Self {
		let vertex_buffer = self.renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: self.label,
			contents: bytemuck::cast_slice(data),
			usage: wgpu::BufferUsages::VERTEX,
		});
		self.vertex_buffers.push(vertex_buffer);
		self
	}

	pub fn vertex_buffer_raw<'b>(&'b mut self, data : &[u8], stride : usize) -> &'b mut Self {
		let vertex_buffer = self.renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: self.label,
			contents: data,
			usage: wgpu::BufferUsages::VERTEX,
		});
		self.vertex_buffers.push(vertex_buffer);
		self
	}

	pub fn index_buffer<'b, T : bytemuck::Pod>(&'b mut self, data : &[T], format : wgpu::IndexFormat) -> &'b mut Self {
		self.index_format = format;
		self.index_buffer = Some(self.renderer.device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: self.label,
				contents: bytemuck::cast_slice(data),
				usage: wgpu::BufferUsages::INDEX,
			}
		));
		self
	}

	pub fn index_buffer_raw<'b>(&'b mut self, data : &[u8], stride : usize, format : wgpu::IndexFormat) -> &'b mut Self {
		self.index_format = format;
		self.index_buffer = Some(self.renderer.device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: self.label,
				contents: data,
				usage: wgpu::BufferUsages::INDEX,
			}
		));
		self
	}

	pub fn submesh<'b>(&'b mut self, vertex_ranges : &[IndexRange], num_elements : u32) -> &'b mut Self {
		self.submeshes.push(SubMesh { 
			vertex_ranges: Vec::from(vertex_ranges),
			index_range: None,
			num_elements: num_elements
		});
		self
	}

	pub fn indexed_submesh<'b>(&'b mut self, vertex_ranges : &[IndexRange], index_range : IndexRange, num_elements : u32) -> &'b mut Self {
		self.submeshes.push(SubMesh { 
			vertex_ranges: Vec::from(vertex_ranges),
			index_range: Some(index_range),
			num_elements: num_elements
		});
		self
	}

	pub fn produce_static_mesh(&mut self) -> StaticMesh {
		StaticMesh {
			vertex_buffers: std::mem::take(&mut self.vertex_buffers),
			index_buffer: std::mem::take(&mut self.index_buffer),
			index_format: self.index_format,
			submeshes: std::mem::take(&mut self.submeshes)
		}
	}
}