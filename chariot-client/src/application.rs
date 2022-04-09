use gltf::Texture;
use specs::{Join, WorldExt};
use std::{
    cmp::Eq,
    collections::HashMap,
    default,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::drawable::*;
use crate::renderer::*;

fn to_wgpu_format(format: gltf::image::Format) -> wgpu::TextureFormat {
    match format {
        gltf::image::Format::R8 => wgpu::TextureFormat::R8Unorm,
        gltf::image::Format::R8G8 => wgpu::TextureFormat::Rg8Unorm,
        gltf::image::Format::R8G8B8 => wgpu::TextureFormat::Etc2Rgb8Unorm,
        gltf::image::Format::R8G8B8A8 => wgpu::TextureFormat::Rgba8Unorm,
        gltf::image::Format::R16 => wgpu::TextureFormat::R16Unorm,
        gltf::image::Format::R16G16 => wgpu::TextureFormat::Rg16Unorm,
        gltf::image::Format::R16G16B16 => panic!("TODO: convert to rgba...."),
        gltf::image::Format::R16G16B16A16 => wgpu::TextureFormat::Rgba16Unorm,
        gltf::image::Format::B8G8R8 => panic!("TODO: convert to rgba..."),
        gltf::image::Format::B8G8R8A8 => wgpu::TextureFormat::Rgba16Unorm,
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct TextureHandle(usize);
#[derive(PartialEq, Eq, Hash)]
pub struct MaterialHandle(usize);
#[derive(PartialEq, Eq, Hash)]
pub struct StaticMeshHandle(usize);

trait Handle {
    fn unique() -> Self;
}

impl Handle for TextureHandle {
    fn unique() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Handle for MaterialHandle {
    fn unique() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Handle for StaticMeshHandle {
    fn unique() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct ResourceManager {
    pub textures: HashMap<TextureHandle, wgpu::Texture>,
    pub materials: HashMap<MaterialHandle, Material>,
    pub meshes: HashMap<StaticMeshHandle, StaticMesh>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            materials: HashMap::new(),
            meshes: HashMap::new(),
        }
    }

    fn import_gltf(
        &mut self,
        renderer: &Renderer,
        filename: &str,
    ) -> core::result::Result<(), gltf::Error> {
        let model_name = filename.split(".").next().expect("invalid filename format");
        let (document, buffers, images) = gltf::import(filename)?;
        let texture_iter = images.iter().map(|img| {
            renderer.create_texture(
                &wgpu::TextureDescriptor {
                    label: None, // TODO: labels
                    size: wgpu::Extent3d {
                        width: img.width,
                        height: img.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: to_wgpu_format(img.format),
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::STORAGE_BINDING,
                },
                &img.pixels,
            )
        });

        let texture_with_id_iter = (0..images.len())
            .map(|idx| TextureHandle::unique())
            .zip(texture_iter);

        for (mesh_idx, mesh) in document.meshes().enumerate() {
            for (prim_idx, primitive) in mesh.primitives().enumerate() {
                let mut mesh_builder = MeshBuilder::new(&renderer, None);
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                if let Some(vert_iter) = reader.read_positions() {
                    mesh_builder.vertex_buffer(bytemuck::cast_slice::<[f32; 3], u8>(
                        &vert_iter.collect::<Vec<[f32; 3]>>(),
                    ));
                }

                if let Some(norm_iter) = reader.read_normals() {
                    mesh_builder.vertex_buffer(bytemuck::cast_slice::<[f32; 3], u8>(
                        &norm_iter.collect::<Vec<[f32; 3]>>(),
                    ));
                }

                if let Some(tc_iter) = reader.read_tex_coords(0) {
                    match tc_iter {
                        gltf::mesh::util::ReadTexCoords::U8(iter) => mesh_builder.vertex_buffer(
                            bytemuck::cast_slice::<[u8; 2], u8>(&iter.collect::<Vec<[u8; 2]>>()),
                        ),
                        gltf::mesh::util::ReadTexCoords::U16(iter) => mesh_builder.vertex_buffer(
                            bytemuck::cast_slice::<[u16; 2], u8>(&iter.collect::<Vec<[u16; 2]>>()),
                        ),
                        gltf::mesh::util::ReadTexCoords::F32(iter) => mesh_builder.vertex_buffer(
                            bytemuck::cast_slice::<[f32; 2], u8>(&iter.collect::<Vec<[f32; 2]>>()),
                        ),
                    };
                }

                let full_range = (
                    std::ops::Bound::<u64>::Unbounded,
                    std::ops::Bound::<u64>::Unbounded,
                );
                let vertex_ranges = vec![full_range; mesh_builder.vertex_buffers.len()];
                if let Some(indices) = reader.read_indices() {
                    let num_elements = match indices {
                        gltf::mesh::util::ReadIndices::U16(iter) => {
                            let tmp_len = iter.len();
                            mesh_builder.index_buffer(
                                bytemuck::cast_slice::<u16, u8>(&iter.collect::<Vec<u16>>()),
                                wgpu::IndexFormat::Uint16,
                            );
                            tmp_len
                        }
                        gltf::mesh::util::ReadIndices::U32(iter) => {
                            let tmp_len = iter.len();
                            mesh_builder.index_buffer(
                                bytemuck::cast_slice::<u32, u8>(&iter.collect::<Vec<u32>>()),
                                wgpu::IndexFormat::Uint32,
                            );
                            tmp_len
                        }
                        _ => panic!("u8 indices????"),
                    };
                    let indices_range = full_range;
                    mesh_builder.indexed_submesh(
                        &vertex_ranges,
                        indices_range,
                        u32::try_from(num_elements).unwrap(),
                    );
                };

                // TODO: unindexed meshes
                //mesh_builder.submesh(&vertex_ranges, num_elements)
                // TODO: rest

                self.meshes.insert(
                    StaticMeshHandle::unique(),
                    mesh_builder.produce_static_mesh(),
                );
            }
        }

        self.textures.extend(texture_with_id_iter);

        core::result::Result::Ok(())
    }
}

struct StaticMeshRenderSystem<'a> {
    renderer: &'a mut Renderer,
    resources: &'a ResourceManager,
}

impl<'a> StaticMeshRenderSystem<'a> {
    fn new(renderer: &'a mut Renderer, resources: &'a ResourceManager) -> Self {
        Self {
            renderer: renderer,
            resources: resources,
        }
    }
}

impl<'a, 'b> specs::System<'b> for StaticMeshRenderSystem<'a> {
    type SystemData = specs::ReadStorage<'b, StaticMeshDrawable>;

    fn run(&mut self, drawables: Self::SystemData) {
        let mut render_job = RenderJob::new();
        for drawable in drawables.join() {
            let render_item = drawable.render_item(self.resources);
            render_job.add_item(render_item);
        }

        self.renderer.render(&render_job);
    }
}

pub struct Application {
    pub world: specs::World,
    pub renderer: Renderer,
    resources: ResourceManager,
}

impl Application {
    pub fn new(renderer: Renderer) -> Self {
        let mut world = specs::World::empty();
        world.register::<StaticMeshDrawable>();
        Self {
            world: world,
            renderer: renderer,
            resources: ResourceManager::new(),
        }
    }

    pub fn render(&mut self) {
        if !self.world.has_value::<StaticMeshDrawable>() {
            return;
        }

        let mut dispatcher = specs::DispatcherBuilder::new()
            .with(
                StaticMeshRenderSystem::new(&mut self.renderer, &self.resources),
                "StaticMeshRenderSystem",
                &[],
            )
            .build();

        dispatcher.dispatch_seq(&self.world);
    }

    pub fn update(&mut self) {}

    // TODO: input handlers
}
