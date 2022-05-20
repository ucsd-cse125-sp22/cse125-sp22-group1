use super::RenderContext;
use super::Technique;
use super::TransformUniform;
use crate::renderer::*;
use crate::resources::*;

pub struct GeometryDrawTechnique {
    material: MaterialHandle,
    static_mesh: StaticMeshHandle,
    submesh_idx: usize,
    pub mvp_xform: TransformUniform<3>,
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
        bind_group_refs.extend(material.bind_groups(context.iteration));
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
        bind_group_refs.extend(material.bind_groups(context.iteration));
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
