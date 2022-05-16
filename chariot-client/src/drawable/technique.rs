use std::ops::Bound;
use std::ops::RangeBounds;

use crate::drawable::*;
use crate::renderer::*;
use crate::resources::*;
use wgpu::util::DeviceExt;
use wgpu::BufferAddress;

pub trait Technique {
    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderItem<'a>;
}

pub(super) struct TransformUniform<const NUM_ELEMS: usize> {
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

pub struct GeometryDrawTechnique {
    material: MaterialHandle,
    static_mesh: StaticMeshHandle,
    submesh_idx: usize,
    pub(super) mvp_xform: TransformUniform<3>,
}

impl GeometryDrawTechnique {
    pub fn new(
        renderer: &Renderer,
        resources: &ResourceManager,
        material: MaterialHandle,
        static_mesh: StaticMeshHandle,
        submesh_idx: usize,
    ) -> Self {
        let pass_name = &resources
            .materials
            .get(&material)
            .expect("invalid material handle")
            .pass_name;

        Self {
            material: material,
            static_mesh: static_mesh,
            submesh_idx: submesh_idx,
            mvp_xform: TransformUniform::new(renderer, pass_name, 0),
        }
    }
}

impl Technique for GeometryDrawTechnique {
    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderItem<'a> {
        let static_mesh = context
            .resources
            .meshes
            .get(&self.static_mesh)
            .expect("invalid static mesh handle");
        let material = context
            .resources
            .materials
            .get(&self.material)
            .expect("invalid material handle");

        let vertex_buffers_with_ranges = static_mesh
            .vertex_buffers
            .iter()
            .zip(static_mesh.submeshes[self.submesh_idx].vertex_ranges.iter());

        let mut bind_group_refs = vec![&self.mvp_xform.bind_group];
        bind_group_refs.extend(material.bind_groups.values());
        render_job::RenderItem::Graphics {
            pass_name: material.pass_name.as_str(),
            framebuffer_name: context.framebuffer_name("geometry_out"),
            num_elements: static_mesh.submeshes[self.submesh_idx].num_elements,
            vertex_buffers: vertex_buffers_with_ranges
                .map(|(buffer, range)| buffer.slice(*range))
                .collect::<Vec<wgpu::BufferSlice>>(),
            index_buffer: match &static_mesh.index_buffer {
                Some(buffer) => {
                    Some(buffer.slice(static_mesh.submeshes[self.submesh_idx].index_range.unwrap()))
                }
                None => None,
            },
            index_format: static_mesh.index_format,
            bind_group: bind_group_refs,
        }
    }
}

pub struct SurfelGeometryDrawTechnique {
    material: MaterialHandle,
    static_mesh: StaticMeshHandle,
    submesh_idx: usize,
    pub(super) mvp_xform: TransformUniform<3>,
}

// For debugging:
impl SurfelGeometryDrawTechnique {
    const PASS_NAME: &'static str = "surfel_geometry";
    const FRAMEBUFFER_NAME: &'static str = "geometry_out";
    pub fn new(
        renderer: &Renderer,
        resources: &ResourceManager,
        material: MaterialHandle,
        static_mesh: StaticMeshHandle,
        submesh_idx: usize,
    ) -> Self {
        let pass_name = &resources
            .materials
            .get(&material)
            .expect("invalid material handle")
            .pass_name;

        Self {
            material: material,
            static_mesh: static_mesh,
            submesh_idx: submesh_idx,
            mvp_xform: TransformUniform::new(renderer, pass_name, 0),
        }
    }
}

impl Technique for SurfelGeometryDrawTechnique {
    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderItem<'a> {
        let static_mesh = context
            .resources
            .meshes
            .get(&self.static_mesh)
            .expect("invalid static mesh handle");
        let material = context
            .resources
            .materials
            .get(&self.material)
            .expect("invalid material handle");

        let vertex_buffers = [
            &static_mesh.surfel_points_buf,
            &static_mesh.surfel_normals_buf,
            &static_mesh.surfel_colors_buf,
        ]
        .iter()
        .filter_map(|maybe_buf| maybe_buf.as_ref().map(|buf| buf.slice(..)))
        .collect::<Vec<wgpu::BufferSlice>>();

        let mut bind_group_refs = vec![&self.mvp_xform.bind_group];
        bind_group_refs.extend(material.bind_groups.values());
        render_job::RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: context.framebuffer_name(Self::FRAMEBUFFER_NAME),
            num_elements: static_mesh.num_surfels,
            vertex_buffers: vertex_buffers,
            index_buffer: None,
            index_format: static_mesh.index_format,
            bind_group: bind_group_refs,
        }
    }
}

pub struct ShadowDrawTechnique {
    pass_name: String,
    framebuffer_name: String,
    static_mesh: StaticMeshHandle,
    submesh_idx: usize,
    pub(super) mvp_xform: TransformUniform<1>,
}

impl ShadowDrawTechnique {
    pub fn new(
        renderer: &Renderer,
        static_mesh: StaticMeshHandle,
        submesh_idx: usize,
        pass_name: &str,
        framebuffer_name: &str,
    ) -> Self {
        Self {
            pass_name: pass_name.to_string(),
            framebuffer_name: framebuffer_name.to_string(),
            static_mesh: static_mesh,
            submesh_idx: submesh_idx,
            mvp_xform: TransformUniform::new(renderer, pass_name, 0),
        }
    }
}

impl Technique for ShadowDrawTechnique {
    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderItem<'a> {
        let static_mesh = context
            .resources
            .meshes
            .get(&self.static_mesh)
            .expect("invalid static mesh handle");

        let vertex_buffers_with_ranges = static_mesh
            .vertex_buffers
            .iter()
            .zip(static_mesh.submeshes[self.submesh_idx].vertex_ranges.iter());

        let bind_group_refs = vec![&self.mvp_xform.bind_group];

        render_job::RenderItem::Graphics {
            pass_name: self.pass_name.as_str(),
            framebuffer_name: context.framebuffer_name(self.framebuffer_name.as_str()),
            num_elements: static_mesh.submeshes[self.submesh_idx].num_elements,
            vertex_buffers: vertex_buffers_with_ranges
                .map(|(buffer, range)| buffer.slice(*range))
                .collect::<Vec<wgpu::BufferSlice>>(),
            index_buffer: match &static_mesh.index_buffer {
                Some(buffer) => {
                    Some(buffer.slice(static_mesh.submeshes[self.submesh_idx].index_range.unwrap()))
                }
                None => None,
            },
            index_format: static_mesh.index_format,
            bind_group: bind_group_refs,
        }
    }
}

fn load_tex(resources: &ResourceManager, name: &str, idx: usize) -> wgpu::TextureView {
    // I don't like the idea of making new views all the time but it's too built into the design now; maybe later I'll fix it
    resources
        .framebuffer_tex(name, idx, false)
        .expect(format!("Technique requires {} framebuffer to be registered", name).as_str())
        .create_view(&wgpu::TextureViewDescriptor::default())
}

fn load_alt_tex(resources: &ResourceManager, name: &str, idx: usize) -> wgpu::TextureView {
    // I don't like the idea of making new views all the time but it's too built into the design now; maybe later I'll fix it
    resources
        .framebuffer_tex(name, idx, true)
        .expect(format!("Technique requires {} framebuffer to be registered", name).as_str())
        .create_view(&wgpu::TextureViewDescriptor::default())
}

pub struct ShadeTechnique {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    material: material::Material,
    view_xform: TransformUniform<2>,
    light_xform: TransformUniform<1>,
}

impl ShadeTechnique {
    pub fn new(renderer: &Renderer, resources: &ResourceManager, pass_name: &str) -> Self {
        let verts_data: [[f32; 2]; 4] = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
        let inds_data: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("fsq_verts"),
                contents: bytemuck::cast_slice(&verts_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("fsq_inds"),
                contents: bytemuck::cast_slice(&inds_data),
                usage: wgpu::BufferUsages::INDEX,
            });

        let color_tex_view = load_tex(resources, "geometry_out", 0);
        let normal_tex_view = load_tex(resources, "geometry_out", 1);
        let depth_tex_view = load_tex(resources, "geometry_out", 2);
        let shadow_tex_view = load_tex(resources, "shadow_out1", 0);
        let probes_color_tex_view = load_tex(resources, "probes_out", 0);
        let probes_depth_tex_view = load_tex(resources, "probes_out", 1);

        let probe_sampler = renderer.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("probe_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let view_xform = TransformUniform::<2>::new(renderer, "shade", 1);
        let light_xform = TransformUniform::<1>::new(renderer, "shade", 2);

        let material = material::MaterialBuilder::new(renderer, pass_name)
            .texture_resource(0, 0, color_tex_view)
            .texture_resource(0, 1, normal_tex_view)
            .texture_resource(0, 2, depth_tex_view)
            .texture_resource(0, 3, shadow_tex_view)
            .texture_resource(0, 4, probes_color_tex_view)
            .texture_resource(0, 5, probes_depth_tex_view)
            .sampler_resource(0, 6, probe_sampler)
            .produce();

        Self {
            vertex_buffer,
            index_buffer,
            material,
            view_xform,
            light_xform,
        }
    }

    pub fn update_view_data(&self, renderer: &Renderer, view: glam::Mat4, proj: glam::Mat4) {
        let inv_view = view.inverse();
        let inv_proj = proj.inverse();
        self.view_xform.update(renderer, &[inv_view, inv_proj]);
    }

    pub fn update_light_data(&self, renderer: &Renderer, view: glam::Mat4, proj: glam::Mat4) {
        let view_proj = proj * view;
        self.light_xform.update(renderer, &[view_proj]);
    }
}

impl Technique for ShadeTechnique {
    fn render_item<'a>(&'a self, _: &RenderContext<'a>) -> render_job::RenderItem<'a> {
        let bind_groups = self
            .material
            .bind_groups
            .values()
            .chain(std::slice::from_ref(&self.view_xform.bind_group).iter())
            .chain(std::slice::from_ref(&self.light_xform.bind_group).iter())
            .collect();

        render_job::RenderItem::Graphics {
            pass_name: self.material.pass_name.as_str(),
            framebuffer_name: "surface".to_string(),
            num_elements: 6,
            vertex_buffers: vec![self.vertex_buffer.slice(..)],
            index_buffer: Some(self.index_buffer.slice(..)),
            index_format: wgpu::IndexFormat::Uint16,
            bind_group: bind_groups,
        }
    }
}

pub struct InitProbesTechnique {
    static_mesh: StaticMeshHandle,
    material: material::Material,
    pub(super) mvp_xform: TransformUniform<4>,
    pub(super) light_xform: TransformUniform<1>,
}

impl InitProbesTechnique {
    const PASS_NAME: &'static str = "init_probes";
    const FRAMEBUFFER_NAME: &'static str = "probes_out";
    pub fn new(
        renderer: &Renderer,
        resources: &ResourceManager,
        static_mesh: StaticMeshHandle,
    ) -> Self {
        let color_tex_view = load_tex(resources, "geometry_out", 0);
        let normal_tex_view = load_tex(resources, "geometry_out", 1);
        let depth_tex_view = load_tex(resources, "geometry_out", 2);
        let shadow_tex_view = load_tex(resources, "shadow_out1", 0);

        let material = material::MaterialBuilder::new(renderer, Self::PASS_NAME)
            .texture_resource(0, 0, color_tex_view)
            .texture_resource(0, 1, normal_tex_view)
            .texture_resource(0, 2, depth_tex_view)
            .texture_resource(0, 3, shadow_tex_view)
            .produce();

        Self {
            static_mesh,
            material,
            mvp_xform: TransformUniform::<4>::new(renderer, Self::PASS_NAME, 1),
            light_xform: TransformUniform::<1>::new(renderer, Self::PASS_NAME, 2),
        }
    }
}

impl Technique for InitProbesTechnique {
    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderItem<'a> {
        let static_mesh = context
            .resources
            .meshes
            .get(&self.static_mesh)
            .expect("invalid static mesh handle");

        const SURFELS_PER_DRAW: u32 = 10_000;
        const BYTES_PER_VERT: u32 = std::mem::size_of::<glam::Vec3>() as u32;
        let (group_range, num_elems) = if static_mesh.num_surfels > 0 {
            let group_idx = context.iteration % ((static_mesh.num_surfels / SURFELS_PER_DRAW) + 1);
            let low_bound = group_idx * SURFELS_PER_DRAW;
            let high_bound =
                std::cmp::min((group_idx + 1) * SURFELS_PER_DRAW, static_mesh.num_surfels);
            (
                (
                    Bound::Included((low_bound * BYTES_PER_VERT) as BufferAddress),
                    Bound::Excluded((high_bound * BYTES_PER_VERT) as BufferAddress),
                ),
                high_bound - low_bound,
            )
        } else {
            ((Bound::Unbounded, Bound::Unbounded), 0)
        };

        let vertex_buffers = [
            &static_mesh.surfel_points_buf,
            &static_mesh.surfel_normals_buf,
            &static_mesh.surfel_colors_buf,
        ]
        .iter()
        .filter_map(|maybe_buf| maybe_buf.as_ref().map(|buf| buf.slice(group_range)))
        .collect::<Vec<wgpu::BufferSlice>>();

        let bind_groups = self
            .material
            .bind_groups
            .values()
            .chain(std::slice::from_ref(&self.mvp_xform.bind_group).iter())
            .chain(std::slice::from_ref(&self.light_xform.bind_group).iter())
            .collect();

        render_job::RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: context.framebuffer_name(Self::FRAMEBUFFER_NAME),
            num_elements: num_elems,
            vertex_buffers: vertex_buffers,
            index_buffer: None,
            index_format: static_mesh.index_format,
            bind_group: bind_groups,
        }
    }
}

pub struct TemporalAccProbesTechnique {
    num_elems: u32,
    material: material::Material,
    alt_material: material::Material,
    pub(super) view_xform: TransformUniform<4>,
}

impl TemporalAccProbesTechnique {
    const PASS_NAME: &'static str = "temporal_acc_probes";
    const FRAMEBUFFER_NAME: &'static str = "probes_acc_out";
    fn probe_sampler(renderer: &Renderer) -> wgpu::Sampler {
        renderer.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("probe_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    }

    pub fn new(renderer: &Renderer, resources: &ResourceManager) -> Self {
        let material = {
            let depth_tex_view = load_tex(resources, "geometry_out", 2);
            let alt_depth_tex_view = load_alt_tex(resources, "geometry_out", 2);

            let alt_probes_color_tex_view = load_alt_tex(resources, "probes_out", 0);
            let alt_probes_depth_tex_view = load_alt_tex(resources, "probes_out", 1);

            material::MaterialBuilder::new(renderer, Self::PASS_NAME)
                .texture_resource(0, 0, depth_tex_view)
                .texture_resource(0, 1, alt_depth_tex_view)
                .texture_resource(0, 2, alt_probes_color_tex_view)
                .texture_resource(0, 3, alt_probes_depth_tex_view)
                .sampler_resource(0, 4, TemporalAccProbesTechnique::probe_sampler(renderer))
                .produce()
        };

        let alt_material = {
            let depth_tex_view = load_tex(resources, "geometry_out", 2);
            let alt_depth_tex_view = load_alt_tex(resources, "geometry_out", 2);

            let probes_color_tex_view = load_tex(resources, "probes_out", 0);
            let probes_depth_tex_view = load_tex(resources, "probes_out", 1);

            material::MaterialBuilder::new(renderer, Self::PASS_NAME)
                .texture_resource(0, 0, alt_depth_tex_view)
                .texture_resource(0, 1, depth_tex_view)
                .texture_resource(0, 2, probes_color_tex_view)
                .texture_resource(0, 3, probes_depth_tex_view)
                .sampler_resource(0, 4, TemporalAccProbesTechnique::probe_sampler(renderer))
                .produce()
        };

        let surface_size = renderer.surface_size();
        let num_elems = surface_size.width * surface_size.height;

        Self {
            num_elems,
            material,
            alt_material,
            view_xform: TransformUniform::<4>::new(renderer, Self::PASS_NAME, 1),
        }
    }
}

impl Technique for TemporalAccProbesTechnique {
    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderItem<'a> {
        let material = if context.iteration % 2 == 0 {
            &self.material
        } else {
            &self.alt_material
        };

        let bind_groups = material
            .bind_groups
            .values()
            .chain(std::slice::from_ref(&self.view_xform.bind_group).iter())
            .collect();

        render_job::RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: context.framebuffer_name(Self::FRAMEBUFFER_NAME),
            num_elements: self.num_elems,
            vertex_buffers: vec![],
            index_buffer: None,
            index_format: wgpu::IndexFormat::Uint16,
            bind_group: bind_groups,
        }
    }
}
