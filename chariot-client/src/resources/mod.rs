use std::{
    cmp::Eq,
    collections::HashMap,
    default,
    sync::atomic::{AtomicUsize, Ordering},
};

pub mod material;
pub mod static_mesh;

use material::*;
use static_mesh::*;
use wgpu::util::DeviceExt;
use chariot_core::GLOBAL_CONFIG;

use crate::drawable::*;
use crate::renderer::*;

// This file has the ResourceManager, which is responsible for loading gltf models and assigning resource handles

fn to_wgpu_format(format: gltf::image::Format) -> wgpu::TextureFormat {
    match format {
        gltf::image::Format::R8 => wgpu::TextureFormat::R8Unorm,
        gltf::image::Format::R8G8 => wgpu::TextureFormat::Rg8Unorm,
        gltf::image::Format::R8G8B8 => wgpu::TextureFormat::Rgba8Unorm, // TODO: this isn't supported on some platforms
        gltf::image::Format::R8G8B8A8 => wgpu::TextureFormat::Rgba8Unorm,
        gltf::image::Format::R16 => wgpu::TextureFormat::R16Unorm,
        gltf::image::Format::R16G16 => wgpu::TextureFormat::Rg16Unorm,
        gltf::image::Format::R16G16B16 => panic!("TODO: convert to rgba...."),
        gltf::image::Format::R16G16B16A16 => wgpu::TextureFormat::Rgba16Unorm,
        gltf::image::Format::B8G8R8 => panic!("TODO: convert to rgba..."),
        gltf::image::Format::B8G8R8A8 => wgpu::TextureFormat::Rgba16Unorm,
    }
}

fn rgb8_to_rgba8(data: &[u8]) -> Vec<u8> {
    let mut res = Vec::new();
    for rgb_data in data.chunks(3) {
        res.extend(rgb_data);
        res.push(255);
    }

    res
}

/*
 * Having direct references to resources can piss off the borrow handler.
 * With the way I have things set up, resources can be loaded and unloaded at any time,
 * so the rust borrow handler can't tell statically when resources come into and out of scope.
 * These resource handles can make things a lot nicer
 */

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct TextureHandle(usize);
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct MaterialHandle(usize);
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
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

pub type Bounds = (glam::Vec3, glam::Vec3);

pub fn new_bounds() -> Bounds {
    let low_bound = glam::vec3(f32::MAX, f32::MAX, f32::MAX);
    let high_bound = glam::vec3(f32::MIN, f32::MIN, f32::MIN);
    (low_bound, high_bound)
}

pub fn accum_bounds(mut acc: Bounds, new: Bounds) -> Bounds {
    acc.0 = acc.0.min(new.0);
    acc.1 = acc.1.max(new.1);
    acc
}

pub struct ImportData {
    pub tex_handles: Vec<TextureHandle>,
    pub material_handles: Vec<MaterialHandle>,
    pub mesh_handles: Vec<StaticMeshHandle>,
    pub drawables: Vec<StaticMeshDrawable>,
    pub bounds: Bounds,
}

pub struct ResourceManager {
    pub framebuffers: HashMap<String, Vec<TextureHandle>>,
    pub textures: HashMap<TextureHandle, wgpu::Texture>,
    pub materials: HashMap<MaterialHandle, Material>,
    pub meshes: HashMap<StaticMeshHandle, StaticMesh>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            framebuffers: HashMap::new(),
            textures: HashMap::new(),
            materials: HashMap::new(),
            meshes: HashMap::new(),
        }
    }

    /*
     * Imports the meshes, textures and (TODO: materials) from a gltf file.
     * Learn more about the gltf file format here: https://www.khronos.org/gltf/
     * It's pretty much the hip open source scene format right now right now for games.
     */
    pub fn import_gltf(
        &mut self,
        renderer: &mut Renderer,
        filename: &str,
    ) -> core::result::Result<ImportData, gltf::Error> {
        println!(
            "loading {}, please give a sec I swear it's not lagging",
            filename
        );
        let model_name = filename.split(".").next().expect("invalid filename format");
        let resource_path = format!("{}/{}", GLOBAL_CONFIG.resource_folder, filename);
        let (document, buffers, images) = gltf::import(resource_path)?;

        let mut bounds = new_bounds();
        let mut mesh_handles = Vec::new();
        for (mesh_idx, mesh) in document.meshes().enumerate() {
            println!(
                "processing mesh {}",
                mesh.name().unwrap_or("[a mesh that's not named]")
            );

            if mesh.primitives().len() != 1 {
                println!(
                    "Warning: I'm expecting one prim per mesh so things might not work properly"
                );
            }

            for (prim_idx, primitive) in mesh.primitives().enumerate() {
                println!("\tprocessing prim {}", prim_idx);
                let (handle, mesh_bounds) = self.import_mesh(renderer, &buffers, &primitive);
                mesh_handles.push(handle);

                bounds = accum_bounds(bounds, mesh_bounds);
            }
        }

        println!("uploading textures...");
        let tex_handles = self.upload_textures(renderer, &images);

        let mut material_handles = Vec::<MaterialHandle>::new();
        for material in document.materials() {
            println!(
                "Processing material {}...",
                material.name().unwrap_or("[unnamed material]")
            );
            let handle = self.import_material(renderer, &tex_handles, &material);
            material_handles.push(handle);
        }

        let mut handle_idx = 0;
        let mut drawables = Vec::<StaticMeshDrawable>::new();
        for (mesh_idx, mesh) in document.meshes().enumerate() {
            for (prim_idx, primitive) in mesh.primitives().enumerate() {
                let material_handle = material_handles[primitive.material().index().unwrap()];
                let mesh_handle = mesh_handles[handle_idx]; // TODO: bug if more than one prim per mesh
                let drawable =
                    StaticMeshDrawable::new(renderer, self, material_handle, mesh_handle, 0);
                drawables.push(drawable);
                handle_idx += 1;
            }
        }

        println!("done!");

        core::result::Result::Ok(ImportData {
            tex_handles,
            material_handles,
            mesh_handles,
            drawables,
            bounds,
        })
    }

    fn import_mesh(
        &mut self,
        renderer: &Renderer,
        buffers: &[gltf::buffer::Data],
        primitive: &gltf::Primitive,
    ) -> (StaticMeshHandle, Bounds) {
        let f32_low = f32::MIN;

        let mut bounds = new_bounds();

        let mut mesh_builder = MeshBuilder::new(renderer, None);
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
        if let Some(vert_iter) = reader.read_positions() {
            let vert_buf = vert_iter.collect::<Vec<[f32; 3]>>();
            let glam_verts = vert_buf.iter().map(|e| glam::Vec3::from_slice(e));

            bounds = accum_bounds(
                bounds,
                (
                    glam_verts.clone().reduce(|a, e| a.min(e)).unwrap(),
                    glam_verts.clone().reduce(|a, e| a.max(e)).unwrap(),
                ),
            );

            mesh_builder.vertex_buffer(&vert_buf);
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

        if mesh_builder.vertex_buffers.len() != 3 {
            println!("unsupported vertex format, your mesh might look weird");
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
        // TODO: rest

        let mesh_handle = StaticMeshHandle::unique();
        self.meshes
            .insert(mesh_handle, mesh_builder.produce_static_mesh());

        (mesh_handle, bounds)
    }

    fn upload_textures(
        &mut self,
        renderer: &Renderer,
        images: &[gltf::image::Data],
    ) -> Vec<TextureHandle> {
        let texture_upload = |img: &gltf::image::Data| {
            let should_expand = img.format == gltf::image::Format::R8G8B8;
            let expanded_img_data = if should_expand {
                rgb8_to_rgba8(&img.pixels)
            } else {
                vec![]
            };
            let img_data = if should_expand {
                &expanded_img_data
            } else {
                &img.pixels
            };
            renderer.create_texture2D_init(
                "tex name",
                winit::dpi::PhysicalSize::<u32> {
                    width: img.width,
                    height: img.height,
                },
                to_wgpu_format(img.format),
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
                &img_data,
            )
        };
        let texture_iter = images.iter().map(texture_upload);

        let handles = (0..images.len())
            .map(|idx| TextureHandle::unique())
            .collect::<Vec<TextureHandle>>();
        self.textures
            .extend(handles.clone().into_iter().zip(texture_iter));

        handles
    }

    // This is still a WIP. Just returns a dummy simple material for now.
    pub fn import_material(
        &mut self,
        renderer: &mut Renderer,
        images: &[TextureHandle],
        material: &gltf::Material,
    ) -> MaterialHandle {
        let pbr_metallic_roughness = material.pbr_metallic_roughness();
        let base_color_index = pbr_metallic_roughness
            .base_color_texture()
            .map(|info| info.texture().source().index())
            .unwrap_or(0); //"No base color tex for material");
        let base_color_handle = images[base_color_index];
        let base_color_view = if let Some(tex_info) = pbr_metallic_roughness.base_color_texture() {
            self.textures
                .get(&base_color_handle)
                .expect("Couldn't find base texture")
                .create_view(&wgpu::TextureViewDescriptor::default())
        } else {
            let bc = pbr_metallic_roughness.base_color_factor();
            let bc_data = [
                (255.0 * bc[0]) as u8,
                (255.0 * bc[1]) as u8,
                (255.0 * bc[2]) as u8,
                (255.0 * bc[3]) as u8,
            ];
            let mat_name = material.name().unwrap_or("unnamed");
            let constant_color_tex = renderer.create_texture2D_init(
                mat_name,
                winit::dpi::PhysicalSize::new(1, 1),
                wgpu::TextureFormat::Rgba8Unorm,
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
                bytemuck::bytes_of(&[bc_data]),
            );
            let tex_handle = TextureHandle::unique();
            self.textures.insert(tex_handle, constant_color_tex);
            self.textures
                .get(&tex_handle)
                .unwrap()
                .create_view(&wgpu::TextureViewDescriptor::default())
        };

        let sampler = renderer.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let material_handle = MaterialHandle::unique();
        let mat_id_buf = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("material_id"),
                contents: bytemuck::bytes_of(&(material_handle.0 as u32)),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let pass_name = "forward";
        let material = MaterialBuilder::new(renderer, pass_name)
            .texture_resource(1, 0, base_color_view)
            .sampler_resource(1, 1, sampler)
            .buffer_resource(1, 2, mat_id_buf)
            .produce();

        self.materials.insert(material_handle, material);
        material_handle
    }

    pub fn depth_framebuffer(
        &mut self,
        name: &str,
        renderer: &Renderer,
        size: winit::dpi::PhysicalSize<u32>,
        formats: &[wgpu::TextureFormat],
        clear_color: Option<wgpu::Color>,
    ) -> FramebufferDescriptor {
        let color_textures: Vec<wgpu::Texture> = formats
            .iter()
            .enumerate()
            .map(|(idx, format)| {
                renderer.create_texture2D(
                    format!("{}_tex_{}", name, idx).as_str(),
                    size,
                    *format,
                    wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::STORAGE_BINDING,
                )
            })
            .collect();

        let color_handles: Vec<TextureHandle> = (0..color_textures.len())
            .map(|idx| TextureHandle::unique())
            .collect();

        let depth_texture = renderer.create_texture2D(
            format!("{}_tex_depth", name).as_str(),
            size,
            Renderer::DEPTH_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );

        let depth_handle = TextureHandle::unique();

        self.framebuffers
            .entry(name.to_string())
            .or_default()
            .extend(color_handles.iter().chain([depth_handle].iter()));

        let desc = FramebufferDescriptor {
            color_attachments: color_textures
                .iter()
                .map(|tex| tex.create_view(&wgpu::TextureViewDescriptor::default()))
                .collect(),
            depth_stencil_attachment: Some(
                depth_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            ),
            clear_color: clear_color,
            clear_depth: true,
        };

        self.textures.extend(
            color_handles
                .into_iter()
                .chain([depth_handle].into_iter())
                .zip(
                    color_textures
                        .into_iter()
                        .chain([depth_texture].into_iter()),
                ),
        );

        desc
    }

    pub fn depth_surface_framebuffer(
        &mut self,
        name: &str,
        renderer: &Renderer,
        formats: &[wgpu::TextureFormat],
        clear_color: Option<wgpu::Color>,
    ) -> FramebufferDescriptor {
        let surface_size = renderer.surface_size();
        self.depth_framebuffer(name, renderer, surface_size, formats, clear_color)
    }

    pub fn framebuffer_tex(&self, name: &str, index: usize) -> Option<&wgpu::Texture> {
        let handle = self.framebuffers.get(&name.to_string())?.get(index)?;
        self.textures.get(&handle)
    }
}
