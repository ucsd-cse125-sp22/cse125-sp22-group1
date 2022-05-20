use super::Bound;
use super::RenderContext;
use super::Technique;
use super::TransformUniform;
use crate::renderer::*;
use crate::resources::*;

pub struct InitProbesTechnique {
    static_mesh: StaticMeshHandle,
    material: material::Material,
    pub mvp_xform: TransformUniform<4>,
    pub light_xform: TransformUniform<1>,
}

impl InitProbesTechnique {
    const PASS_NAME: &'static str = "init_probes";
    const FRAMEBUFFER_NAME: &'static str = "probes_out";
    pub fn new(
        renderer: &Renderer,
        resources: &ResourceManager,
        static_mesh: StaticMeshHandle,
    ) -> Self {
        let material = material::MaterialBuilder::new(renderer, resources, Self::PASS_NAME)
            .framebuffer_texture_resource(0, 0, "geometry_out", 0, false)
            .framebuffer_texture_resource(0, 1, "geometry_out", 1, false)
            .framebuffer_texture_resource(0, 2, "geometry_out", 2, false)
            .framebuffer_texture_resource(0, 3, "shadow_out1", 0, false)
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
            let low_bound =
                (group_idx * SURFELS_PER_DRAW + context.iteration) % static_mesh.num_surfels;
            let high_bound = std::cmp::min(low_bound + SURFELS_PER_DRAW, static_mesh.num_surfels);
            (
                (
                    Bound::Included((low_bound * BYTES_PER_VERT) as wgpu::BufferAddress),
                    Bound::Excluded((high_bound * BYTES_PER_VERT) as wgpu::BufferAddress),
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
            .bind_groups(context.iteration)
            .into_iter()
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
    pub view_xform: TransformUniform<4>,
}

impl TemporalAccProbesTechnique {
    const PASS_NAME: &'static str = "temporal_acc_probes";
    const FRAMEBUFFER_NAME: &'static str = "probes_out";
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
        let material = material::MaterialBuilder::new(renderer, resources, Self::PASS_NAME)
            .framebuffer_texture_resource(0, 0, "geometry_out", 2, false)
            .framebuffer_texture_resource(0, 1, "geometry_out", 2, true)
            .framebuffer_texture_resource(0, 2, "probes_out", 0, true)
            .framebuffer_texture_resource(0, 3, "probes_out", 1, true)
            .sampler_resource(0, 4, TemporalAccProbesTechnique::probe_sampler(renderer))
            .produce();

        let surface_size = renderer.surface_size();
        let num_elems = surface_size.width * surface_size.height;

        Self {
            num_elems,
            material,
            view_xform: TransformUniform::<4>::new(renderer, Self::PASS_NAME, 1),
        }
    }

    pub fn update_view_data(
        &self,
        renderer: &Renderer,
        view: glam::Mat4,
        proj: glam::Mat4,
        prev_view: glam::Mat4,
        prev_proj: glam::Mat4,
    ) {
        let inv_view = view.inverse();
        let inv_proj = proj.inverse();
        let prev_inv_view = prev_view.inverse();
        let prev_inv_proj = prev_proj.inverse();
        self.view_xform.update(
            renderer,
            &[inv_view, inv_proj, prev_inv_view, prev_inv_proj],
        );
    }
}

impl Technique for TemporalAccProbesTechnique {
    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderItem<'a> {
        let bind_groups = self
            .material
            .bind_groups(context.iteration)
            .into_iter()
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

pub struct GeometryAccProbesTechnique {
    num_elems: u32,
    material: material::Material,
    pub view_xform: TransformUniform<2>,
}

impl GeometryAccProbesTechnique {
    const PASS_NAME: &'static str = "geometry_acc_probes";
    const FRAMEBUFFER_NAME: &'static str = "probes_out";

    pub fn new(renderer: &Renderer, resources: &ResourceManager) -> Self {
        let material = material::MaterialBuilder::new(renderer, resources, Self::PASS_NAME)
            .framebuffer_texture_resource(0, 0, "geometry_out", 2, false)
            .framebuffer_texture_resource(0, 1, "geometry_out", 0, false)
            .produce();

        let surface_size = renderer.surface_size();
        let num_elems = surface_size.width * surface_size.height;

        Self {
            num_elems,
            material,
            view_xform: TransformUniform::<2>::new(renderer, Self::PASS_NAME, 1),
        }
    }

    pub fn update_view_data(&self, renderer: &Renderer, view: glam::Mat4, proj: glam::Mat4) {
        let inv_view = view.inverse();
        let inv_proj = proj.inverse();
        self.view_xform.update(renderer, &[inv_view, inv_proj]);
    }
}

impl Technique for GeometryAccProbesTechnique {
    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderItem<'a> {
        let bind_groups = self
            .material
            .bind_groups(context.iteration)
            .into_iter()
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
