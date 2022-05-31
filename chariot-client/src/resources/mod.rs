use std::borrow::BorrowMut;
use std::{
    cmp::Eq,
    collections::{HashMap, VecDeque},
    default,
    num::Wrapping,
    ops::Bound,
    sync::atomic::{AtomicUsize, Ordering},
};

use serde_json::Value;
use wgpu::Texture;

pub mod glyph_cache;
pub mod material;
pub mod static_mesh;

use chariot_core::GLOBAL_CONFIG;
use gltf::json::extensions::mesh;
use material::*;
use static_mesh::*;
use wgpu::util::DeviceExt;

use crate::renderer::*;
use crate::resources::glyph_cache::{FontSelection, GlyphCache};
use crate::util::{Pcg32Rng, Rng};
use crate::{drawable::*, scenegraph::components::Modifiers};

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

pub trait Handle {
    const INVALID: Self;
    fn unique() -> Self;
}

impl Handle for TextureHandle {
    const INVALID: Self = TextureHandle(usize::MAX);
    fn unique() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Handle for MaterialHandle {
    const INVALID: Self = MaterialHandle(usize::MAX);
    fn unique() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Handle for StaticMeshHandle {
    const INVALID: Self = StaticMeshHandle(usize::MAX);
    fn unique() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

fn sample_surfels(
    rng: &mut Pcg32Rng,
    verts: &[glam::Vec3],
    tex_coords: &[[f32; 2]],
    inds: &[u32],
    color_data: Option<&BaseColorData>,
    images: &[gltf::image::Data],
    sample_density: f64, // samples / area unit
) -> (Vec<glam::Vec3>, Vec<glam::Vec3>, Vec<glam::Vec3>) {
    let mut points = vec![];
    let mut normals = vec![];
    let mut colors = vec![];

    // TODO: hard-coded for now
    let cd = color_data.unwrap_or(&BaseColorData::Color([255, 255, 255, 255]));
    for tri in inds.chunks(3) {
        let (i, j, k) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let (pi, pj, pk) = (verts[i], verts[j], verts[k]);
        let (tci, tcj, tck) = (
            glam::Vec2::from_slice(&tex_coords[i]),
            glam::Vec2::from_slice(&tex_coords[j]),
            glam::Vec2::from_slice(&tex_coords[k]),
        );
        let (eij, eik) = (pj - pi, pk - pi);

        let Nijk = eij.cross(eik).normalize();
        let Aijk = (eij.cross(eik).length() * 0.5) as f64;

        let u: f32 = rng.next();
        let extra = if f64::fract(Aijk * sample_density) > (u as f64) {
            1usize
        } else {
            0usize
        };
        let samples = (sample_density * Aijk) as usize + extra;

        let (new_points, new_colors): (Vec<glam::Vec3>, Vec<glam::Vec3>) = (0..samples)
            .map(|_| {
                let (mut u0, mut u1): (f32, f32) = (rng.next(), rng.next());
                if u0 + u1 > 1.0 {
                    u0 = 1.0 - u0;
                    u1 = 1.0 - u1;
                }

                let u2 = 1.0 - u0 - u1;
                let tc = tci * u0 + tcj * u1 + tck * u2;
                let color = cd.sample(images, tc);

                (pi + eij * u0 + eik * u1, color)
            })
            .unzip();

        let new_normals = vec![Nijk; samples];

        points.extend(new_points);
        normals.extend(new_normals);
        colors.extend(new_colors);
    }

    println!("\t\tGenerated {} surfel samples", points.len());
    (points, normals, colors)
}

fn sample_image(image: &gltf::image::Data, tc: glam::Vec2) -> [u8; 4] {
    let size = glam::uvec2(image.width, image.height);
    let sizef = size.as_vec2();
    let xyf = tc * sizef;
    let xy = xyf
        .as_uvec2()
        .clamp(glam::UVec2::ZERO, size - glam::UVec2::ONE);
    match image.format {
        gltf::image::Format::R8G8B8 => {
            let idx = ((xy[1] * size[0] + xy[0]) * 3) as usize;
            [
                image.pixels[idx],
                image.pixels[idx + 1],
                image.pixels[idx + 2],
                255,
            ]
        }
        gltf::image::Format::R8G8B8A8 => {
            let idx = ((xy[1] * size[0] + xy[0]) * 4) as usize;
            if idx >= image.pixels.len() {
                println!("{}, {}", xy[0], xy[1]);
                println!("{}, {}", size[0], size[1]);
            }

            [
                image.pixels[idx],
                image.pixels[idx + 1],
                image.pixels[idx + 2],
                255,
            ]
        }
        _ => panic!("TODO: trying to sample unimplemented format"),
    }
}

pub enum BaseColorData {
    ImageIndex(usize),
    Color([u8; 4]),
}

impl BaseColorData {
    fn sample(&self, images: &[gltf::image::Data], tc: glam::Vec2) -> glam::Vec3 {
        let pixel = match self {
            Self::ImageIndex(i) => {
                let img = &images[*i];
                sample_image(img, tc)
            }
            Self::Color(color) => *color,
        };

        glam::vec3(pixel[0] as f32, pixel[1] as f32, pixel[2] as f32) / 256.0
    }
}

pub struct ImportData {
    pub tex_handles: Vec<TextureHandle>,
    pub mesh_handles: Vec<StaticMeshHandle>,
    pub drawables: Vec<StaticMeshDrawable>,
    pub bounds: Bounds,
}

pub struct ResourceManager {
    pub framebuffers: HashMap<String, Vec<TextureHandle>>,
    pub alt_framebuffers: HashMap<String, Vec<TextureHandle>>,
    pub textures: HashMap<TextureHandle, wgpu::Texture>,
    pub materials: HashMap<MaterialHandle, Material>,
    pub meshes: HashMap<StaticMeshHandle, StaticMesh>,
    pub glyph_caches: HashMap<FontSelection, GlyphCache>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            framebuffers: HashMap::new(),
            alt_framebuffers: HashMap::new(),
            textures: HashMap::new(),
            materials: HashMap::new(),
            meshes: HashMap::new(),
            glyph_caches: HashMap::new(),
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
        filename: String,
    ) -> Result<ImportData, gltf::Error> {
        println!(
            "loading {}, please give a sec I swear it's not lagging",
            filename
        );
        let _model_name = filename.split(".").next().expect("invalid filename format");
        let resource_path = format!("{}/{}", GLOBAL_CONFIG.resource_folder, filename);
        let (document, buffers, images) = gltf::import(resource_path)?;
        if document.scenes().count() != 1 {
            panic!(
                "Document {} has {} scenes!",
                filename,
                document.scenes().count()
            );
        }

        let mut bounds = new_bounds();
        let mut mesh_handles = Vec::new();
        let tex_handles = self.upload_textures(renderer, &images);
        let mut material_handles = HashMap::<usize, (MaterialHandle, BaseColorData)>::new();
        let mut drawables = Vec::<StaticMeshDrawable>::new();

        // TODO: Make a real default material, instead of just using the first one
        let (default_material_handle, _) = self.import_material(
            renderer,
            &tex_handles,
            &document.materials().next().unwrap(),
        );

        // Queue of (Node, Transformation) tuples
        let mut queue: VecDeque<(gltf::Node, glam::Mat4)> = document
            .scenes()
            .next()
            .expect("No root node in scene")
            .nodes()
            .map(|n| (n, glam::Mat4::IDENTITY))
            .collect::<VecDeque<(gltf::Node, glam::Mat4)>>();

        let mut n = 0;
        let mut rng = Pcg32Rng::default();
        // Probably better to do this recursively but i didn't wanna change stuff like crazy, not that it really matters since this is just loading anyways
        while let Some((node, parent_transform)) = queue.pop_front() {
            println!("Processing node '{}'", node.name().unwrap_or("<unnamed>"));
            println!("#{} out of #{}", n, queue.capacity());
            n += 1;
            let transform = parent_transform
                * (match node.transform() {
                    gltf::scene::Transform::Matrix { matrix } => {
                        glam::Mat4::from_cols_array_2d(&matrix)
                    }
                    gltf::scene::Transform::Decomposed {
                        translation,
                        rotation,
                        scale,
                    } => glam::Mat4::from_scale_rotation_translation(
                        glam::Vec3::from(scale),
                        glam::Quat::from_array(rotation),
                        glam::Vec3::from(translation),
                    ),
                });

            if let Some(mesh) = node.mesh() {
                let mut render = true;
                if let Some(extras) = mesh.extras().as_ref() {
                    let mesh_data: Value = serde_json::from_str(extras.as_ref().get()).unwrap();
                    if mesh_data["render"] == 0 {
                        println!("\tskipping mesh '{}'", mesh.name().unwrap_or("<unnamed>"));
                        render = false;
                    }
                }

                if render {
                    println!("\tprocessing mesh '{}'", mesh.name().unwrap_or("<unnamed>"));
                    for (prim_idx, primitive) in mesh.primitives().enumerate() {
                        //println!("\t\tprocessing prim {}", prim_idx);
                        //let (mesh_handle, mesh_bounds) =
                        //    self.import_mesh(renderer, &buffers, &primitive, transform);

                        let mut material_handle: &MaterialHandle = &default_material_handle;

                        if let Some(material_id) = primitive.material().index() {
                            material_handle = match material_handles.get(&material_id) {
                                Some((h, _)) => {
                                    // println!("\t\t\tReusing loaded material '{}'", primitive.material().name().unwrap_or("<unnamed>"));
                                    h
                                }
                                None => {
                                    println!(
                                        "\t\t\tProcessing material '{}'...",
                                        primitive.material().name().unwrap_or("<unnamed>")
                                    );
                                    material_handles.insert(
                                        material_id,
                                        self.import_material(
                                            renderer,
                                            &tex_handles,
                                            &primitive.material(),
                                        ),
                                    );
                                    let (h, _) = material_handles.get(&material_id).unwrap();
                                    h
                                }
                            };
                        } else {
                            println!(
                                "Warning: Primitive {}.{} has no material. Using default instead",
                                mesh.name().unwrap_or("<unnamed>"),
                                prim_idx
                            );
                        }

                        let mut modifiers: Modifiers = Default::default();
                        if let Some(extras) = mesh.extras().as_ref() {
                            let mesh_data: Value =
                                serde_json::from_str(extras.as_ref().get()).unwrap();
                            if mesh_data["spin"] == "none" {
                                println!(
                                    "\t\tmesh '{}' will ignore rotation!",
                                    mesh.name().unwrap_or("<unnamed>")
                                );
                                modifiers.absolute_angle = true;
                            }
                        }

                        /*let mut drawable = StaticMeshDrawable::new(
                            renderer,
                            self,
                            *material_handle,
                            mesh_handle,
                            0,
                        );
                        drawable.modifiers = modifiers;
                        drawables.push(drawable);*/

                        let maybe_color_data = primitive
                            .material()
                            .index()
                            .map(|material_id| &material_handles.get(&material_id).unwrap().1);
                        let (mesh_handle, mesh_bounds) = self.import_mesh(
                            renderer,
                            &mut rng,
                            &buffers,
                            &primitive,
                            transform,
                            maybe_color_data,
                            &images,
                        );

                        let drawable = StaticMeshDrawable::new(
                            renderer,
                            self,
                            *material_handle,
                            mesh_handle,
                            0,
                        );
                        drawables.push(drawable);

                        mesh_handles.push(mesh_handle);
                        bounds = accum_bounds(bounds, mesh_bounds);
                    }
                }
            } else {
                println!(
                    "Node '{}' is not a mesh",
                    node.name().unwrap_or("<unnamed>"),
                );
            }

            for child in node.children() {
                queue.push_back((child, transform));
            }
        }

        println!("done!");

        Ok(ImportData {
            tex_handles,
            mesh_handles,
            drawables,
            bounds,
        })
    }

    fn import_mesh(
        &mut self,
        renderer: &Renderer,
        rng: &mut Pcg32Rng,
        buffers: &[gltf::buffer::Data],
        primitive: &gltf::Primitive,
        transform: glam::Mat4,
        color_data: Option<&BaseColorData>,
        images: &[gltf::image::Data],
    ) -> (StaticMeshHandle, Bounds) {
        let _f32_low = f32::MIN;

        let mut bounds = new_bounds();
        let mut vert_vec = None;
        let mut tc_vec = None;

        let mut mesh_builder = MeshBuilder::new(renderer, None);
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
        if let Some(vert_iter) = reader.read_positions() {
            let mut vert_buf = vert_iter.collect::<Vec<[f32; 3]>>();

            for vertex in vert_buf.iter_mut() {
                *vertex = transform
                    .transform_point3(glam::Vec3::from_slice(vertex))
                    .to_array();
            }

            let glam_verts = vert_buf.iter().map(|e| glam::Vec3::from_slice(e));
            vert_vec = Some(glam_verts.clone().collect::<Vec<glam::Vec3>>());

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
                &norm_iter
                    .collect::<Vec<[f32; 3]>>()
                    .iter()
                    .map(|n| {
                        transform
                            .inverse()
                            .transpose()
                            .transform_vector3(glam::Vec3::from_slice(n))
                            .normalize()
                            .to_array()
                    })
                    .collect::<Vec<[f32; 3]>>(),
            ));
        }

        if let Some(tc_iter) = reader.read_tex_coords(0) {
            match tc_iter.clone() {
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

            tc_vec = Some(tc_iter.into_f32().collect::<Vec<[f32; 2]>>());
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
            let num_elements = match indices.clone() {
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

            if vert_vec.is_some() && tc_vec.is_some() && false {
                let (points, normals, colors) = sample_surfels(
                    rng,
                    &vert_vec.unwrap(),
                    &tc_vec.unwrap(),
                    &indices.clone().into_u32().collect::<Vec<u32>>(),
                    color_data,
                    images,
                    10.0,
                );

                mesh_builder.surfels(&points, &normals, &colors);
            } else {
                println!("Warning: skipping surfel generation for model");
            }
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
            renderer.create_texture2d_init(
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
            .map(|_idx| TextureHandle::unique())
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
    ) -> (MaterialHandle, BaseColorData) {
        let pbr_metallic_roughness = material.pbr_metallic_roughness();
        let base_color_index = pbr_metallic_roughness
            .base_color_texture()
            .map(|info| info.texture().source().index())
            .unwrap_or(0); //"No base color tex for material");
        let base_color_handle = images[base_color_index];
        let (base_color_view, base_color_data) =
            if let Some(tex_info) = pbr_metallic_roughness.base_color_texture() {
                (
                    self.textures
                        .get(&base_color_handle)
                        .expect("Couldn't find base texture")
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                    BaseColorData::ImageIndex(base_color_index),
                )
            } else {
                let bc = pbr_metallic_roughness.base_color_factor();
                let bc_data = [
                    (255.0 * bc[0]) as u8,
                    (255.0 * bc[1]) as u8,
                    (255.0 * bc[2]) as u8,
                    (255.0 * bc[3]) as u8,
                ];
                let mat_name = material.name().unwrap_or("unnamed");
                let constant_color_tex = renderer.create_texture2d_init(
                    mat_name,
                    winit::dpi::PhysicalSize::new(1, 1),
                    wgpu::TextureFormat::Rgba8Unorm,
                    wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
                    bytemuck::bytes_of(&[bc_data]),
                );
                let tex_handle = TextureHandle::unique();
                self.textures.insert(tex_handle, constant_color_tex);
                (
                    self.textures
                        .get(&tex_handle)
                        .unwrap()
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                    BaseColorData::Color(bc_data),
                )
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

        let pass_name = "geometry";
        let material = MaterialBuilder::new(renderer, self, pass_name)
            .texture_resource(2, 0, base_color_view)
            .sampler_resource(2, 1, sampler)
            .buffer_resource(2, 2, mat_id_buf)
            .produce();

        self.materials.insert(material_handle, material);
        (material_handle, base_color_data)
    }

    fn create_framebuffer_textures(
        &mut self,
        name: &str,
        renderer: &mut Renderer,
        size: winit::dpi::PhysicalSize<u32>,
        formats: &[wgpu::TextureFormat],
        clear_color: Option<wgpu::Color>,
        is_alt: bool,
    ) -> Vec<TextureHandle> {
        let color_textures: Vec<wgpu::Texture> = formats
            .iter()
            .enumerate()
            .map(|(idx, format)| {
                renderer.create_texture2d(
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
            .map(|_idx| TextureHandle::unique())
            .collect();

        let depth_handle = TextureHandle::unique();

        let desc = FramebufferDescriptor {
            color_attachments: color_textures
                .iter()
                .map(|tex| tex.create_view(&wgpu::TextureViewDescriptor::default()))
                .collect(),
            depth_stencil_attachment: None,
            clear_color: clear_color,
            clear_depth: false,
        };

        let register_name = format!("{}.{}", name, if is_alt { 1 } else { 0 });
        renderer.register_framebuffer(register_name.as_str(), desc);

        self.textures.extend(
            color_handles
                .clone()
                .into_iter()
                .chain([depth_handle].into_iter())
                .zip(color_textures.into_iter()),
        );

        color_handles
    }

    fn create_depth_framebuffer_textures(
        &mut self,
        name: &str,
        renderer: &mut Renderer,
        size: winit::dpi::PhysicalSize<u32>,
        formats: &[wgpu::TextureFormat],
        clear_color: Option<wgpu::Color>,
        clear_depth: bool,
        is_alt: bool,
    ) -> (TextureHandle, Vec<TextureHandle>) {
        let depth_texture = renderer.create_texture2d(
            format!("{}_tex_depth", name).as_str(),
            size,
            Renderer::DEPTH_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );

        let color_textures: Vec<wgpu::Texture> = formats
            .iter()
            .enumerate()
            .map(|(idx, format)| {
                renderer.create_texture2d(
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
            .map(|_idx| TextureHandle::unique())
            .collect();

        let depth_handle = TextureHandle::unique();

        let desc = FramebufferDescriptor {
            color_attachments: color_textures
                .iter()
                .map(|tex| tex.create_view(&wgpu::TextureViewDescriptor::default()))
                .collect(),
            depth_stencil_attachment: Some(
                depth_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            ),
            clear_color: clear_color,
            clear_depth: clear_depth,
        };

        let register_name = format!("{}.{}", name, if is_alt { 1 } else { 0 });
        renderer.register_framebuffer(register_name.as_str(), desc);

        self.textures.extend(
            color_handles
                .clone()
                .into_iter()
                .chain([depth_handle].into_iter())
                .zip(
                    color_textures
                        .into_iter()
                        .chain([depth_texture].into_iter()),
                ),
        );

        (depth_handle, color_handles)
    }

    pub fn register_framebuffer(
        &mut self,
        name: &str,
        renderer: &mut Renderer,
        size: winit::dpi::PhysicalSize<u32>,
        formats: &[wgpu::TextureFormat],
        clear_color: Option<wgpu::Color>,
        save_prev: bool,
    ) {
        let color_handles =
            self.create_framebuffer_textures(name, renderer, size, formats, clear_color, false);

        self.framebuffers
            .entry(name.to_string())
            .or_default()
            .extend(color_handles.iter());

        if save_prev {
            let alt_color_handles =
                self.create_framebuffer_textures(name, renderer, size, formats, clear_color, true);

            self.alt_framebuffers
                .entry(name.to_string())
                .or_default()
                .extend(alt_color_handles.iter());
        }
    }

    pub fn register_depth_framebuffer(
        &mut self,
        name: &str,
        renderer: &mut Renderer,
        size: winit::dpi::PhysicalSize<u32>,
        formats: &[wgpu::TextureFormat],
        clear_color: Option<wgpu::Color>,
        clear_depth: bool,
        save_prev: bool,
    ) {
        let (depth_handle, color_handles) = self.create_depth_framebuffer_textures(
            name,
            renderer,
            size,
            formats,
            clear_color,
            clear_depth,
            false,
        );

        self.framebuffers
            .entry(name.to_string())
            .or_default()
            .extend(color_handles.iter().chain([depth_handle].iter()));

        if save_prev {
            let (alt_depth_handle, alt_color_handles) = self.create_depth_framebuffer_textures(
                name,
                renderer,
                size,
                formats,
                clear_color,
                clear_depth,
                true,
            );

            self.alt_framebuffers
                .entry(name.to_string())
                .or_default()
                .extend(alt_color_handles.iter().chain([alt_depth_handle].iter()));
        }
    }

    pub fn register_surface_framebuffer(
        &mut self,
        name: &str,
        renderer: &mut Renderer,
        formats: &[wgpu::TextureFormat],
        clear_color: Option<wgpu::Color>,
        save_prev: bool,
    ) {
        let surface_size = renderer.surface_size();
        self.register_framebuffer(
            name,
            renderer,
            surface_size,
            formats,
            clear_color,
            save_prev,
        )
    }

    pub fn register_depth_surface_framebuffer(
        &mut self,
        name: &str,
        renderer: &mut Renderer,
        formats: &[wgpu::TextureFormat],
        clear_color: Option<wgpu::Color>,
        clear_depth: bool,
        save_prev: bool,
    ) {
        let surface_size = renderer.surface_size();
        self.register_depth_framebuffer(
            name,
            renderer,
            surface_size,
            formats,
            clear_color,
            clear_depth,
            save_prev,
        )
    }

    #[allow(dead_code)]
    pub fn import_texture(&mut self, renderer: &Renderer, filename: &str) -> TextureHandle {
        let tex_name = filename.split(".").next().expect("invalid filename format");
        let resource_path = format!("{}/{}", GLOBAL_CONFIG.resource_folder, filename);
        let img = image::open(resource_path.clone())
            .expect(format!("didn't find {}", resource_path.clone()).as_str());
        let img_rgba8 = img.into_rgba8();

        let texture = renderer.create_texture2d_init(
            tex_name,
            winit::dpi::PhysicalSize::<u32> {
                width: img_rgba8.width(),
                height: img_rgba8.height(),
            },
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            &img_rgba8.into_raw(),
        );

        self.register_texture(texture)
    }

    // shorthand for registering a texture
    pub fn register_texture(&mut self, texture: Texture) -> TextureHandle {
        let handle = TextureHandle::unique();
        self.textures.insert(handle, texture);
        return handle;
    }

    pub fn register_material(&mut self, material: Material) -> MaterialHandle {
        let handle = MaterialHandle::unique();
        self.materials.insert(handle, material);
        return handle;
    }

    pub fn create_quad_mesh(&mut self, renderer: &Renderer) -> StaticMeshHandle {
        let verts_data: [[f32; 2]; 4] = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
        let texcoord_data: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let inds_data: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let full_range = (Bound::<u64>::Unbounded, Bound::<u64>::Unbounded);

        let mesh = MeshBuilder::new(renderer, Some("quad"))
            .vertex_buffer(&verts_data)
            .vertex_buffer(&texcoord_data)
            .index_buffer(&inds_data, wgpu::IndexFormat::Uint16)
            .indexed_submesh(&[full_range, full_range], full_range, 6)
            .produce_static_mesh();

        let mesh_handle = StaticMeshHandle::unique();
        self.meshes.insert(mesh_handle, mesh);
        mesh_handle
    }

    // fetch the glyph_cache for a particular font selection
    pub fn get_glyph_cache(&mut self, font_selection: FontSelection) -> &mut GlyphCache {
        self.glyph_caches
            .entry(font_selection)
            .or_insert_with_key(|selection| GlyphCache::new(selection))
    }

    pub fn framebuffer_tex(&self, name: &str, index: usize, alt: bool) -> Option<&wgpu::Texture> {
        let handle = if !alt {
            self.framebuffers.get(&name.to_string())?.get(index)?
        } else {
            self.alt_framebuffers.get(&name.to_string())?.get(index)?
        };
        self.textures.get(&handle)
    }

    pub fn framebuffer_name(&self, name: &str, alt: bool) -> String {
        let get_alt = alt && self.alt_framebuffers.contains_key(&name.to_string());
        format!("{}.{}", name, if get_alt { 1 } else { 0 })
    }
}
