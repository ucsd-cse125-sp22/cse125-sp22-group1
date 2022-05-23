use super::Drawable;
use super::TransformUniform;
use crate::renderer::render_job;
use crate::renderer::Renderer;
use crate::resources::{Handle, MaterialHandle, ResourceManager, StaticMeshHandle};

// TODO: merge this with the forward pass and make this write depth. I got it to work but it was weird since on certain frames
// the particles wouldn't show up and the outline effect was being applied to the quads

pub struct ParticleDrawable {
    static_mesh_handle: StaticMeshHandle,
    material_handle: MaterialHandle,
    mvp_xform: TransformUniform<2>,
}

impl ParticleDrawable {
    const PASS_NAME: &'static str = "particle";
    const FRAMEBUFFER_NAME: &'static str = "surface_nodepth";
    pub fn new(
        renderer: &Renderer,
        static_mesh_handle: StaticMeshHandle,
        material_handle: MaterialHandle,
    ) -> Self {
        Self {
            static_mesh_handle,
            material_handle,
            mvp_xform: TransformUniform::new(renderer, Self::PASS_NAME, 0),
        }
    }
}

impl ParticleDrawable {
    pub fn update_mvp(
        &self,
        renderer: &Renderer,
        model: glam::Mat4,
        view: glam::Mat4,
        proj: glam::Mat4,
    ) {
        let view_proj = proj * view;
        self.mvp_xform.update(renderer, &[model, view_proj]);
    }
}

impl Drawable for ParticleDrawable {
    fn render_graph<'a>(&'a self, resources: &'a ResourceManager) -> render_job::RenderGraph<'a> {
        let static_mesh = resources
            .meshes
            .get(&self.static_mesh_handle)
            .expect("invalid static mesh handle");
        let material = resources
            .materials
            .get(&self.material_handle)
            .expect("invalid material handle");

        let vertex_buffers_with_ranges = static_mesh
            .vertex_buffers
            .iter()
            .zip(static_mesh.submeshes[0].vertex_ranges.iter());

        let vertex_buffers = vertex_buffers_with_ranges
            .map(|(buffer, range)| buffer.slice(*range))
            .collect::<Vec<wgpu::BufferSlice>>();

        let mut bind_group_refs = vec![&self.mvp_xform.bind_group];
        bind_group_refs.extend(material.bind_groups.values());
        render_job::RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: Self::FRAMEBUFFER_NAME,
            num_elements: static_mesh.submeshes[0].num_elements,
            vertex_buffers,
            index_buffer: match &static_mesh.index_buffer {
                Some(buffer) => Some(buffer.slice(static_mesh.submeshes[0].index_range.unwrap())),
                None => None,
            },
            index_format: static_mesh.index_format,
            bind_group: bind_group_refs,
        }
        .to_graph()
    }
}
