use super::RenderContext;
use super::Technique;
use super::TransformUniform;
use crate::renderer::*;
use crate::resources::*;

pub struct ShadowDrawTechnique {
    pass_name: String,
    framebuffer_name: String,
    static_mesh: StaticMeshHandle,
    submesh_idx: usize,
    pub mvp_xform: TransformUniform<1>,
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
