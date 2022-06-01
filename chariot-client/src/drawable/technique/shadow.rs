use super::RenderContext;
use super::Technique;
use crate::assets::shaders;
use crate::drawable::util::TransformUniform;
use crate::renderer::*;
use crate::resources::*;

mod shadow_draw_technique {
    use crate::drawable::util::TransformUniform;
    use once_cell::sync::OnceCell;
    pub static LIGHT_VIEW_PROJS: OnceCell<Vec<TransformUniform<1>>> = OnceCell::new();
}
pub struct ShadowDrawTechnique {
    framebuffer_name: String,
    static_mesh: StaticMeshHandle,
    submesh_idx: usize,
    pub model_xform: TransformUniform<1>,
}

impl ShadowDrawTechnique {
    pub fn new(
        renderer: &Renderer,
        static_mesh: StaticMeshHandle,
        submesh_idx: usize,
        framebuffer_name: &str,
    ) -> Self {
        Self {
            framebuffer_name: framebuffer_name.to_string(),
            static_mesh: static_mesh,
            submesh_idx: submesh_idx,
            model_xform: TransformUniform::new(renderer, Self::PASS_NAME, 0),
        }
    }

    fn update_model_xform(&self, renderer: &Renderer, model: glam::Mat4) {
        self.model_xform.update(renderer, &[model]);
    }
}

impl Technique for ShadowDrawTechnique {
    const PASS_NAME: &'static str = "shadow";
    fn register(renderer: &mut Renderer) {
        renderer.register_pass(Self::PASS_NAME, &util::shadow_pass!(&shaders::SHADOW));
    }

    fn update_once(renderer: &Renderer, context: &RenderContext) {
        let light_ufms = shadow_draw_technique::LIGHT_VIEW_PROJS.get_or_init(|| {
            let mut light_ufms = vec![];
            for _ in context.light_vps.iter() {
                light_ufms.push(TransformUniform::new(renderer, Self::PASS_NAME, 0));
            }
            light_ufms
        });

        if context.light_vps.len() != light_ufms.len() {
            panic!("number of lights changed! and i dont support that");
        }

        for (idx, (light_view, light_proj)) in context.light_vps.iter().enumerate() {
            let view_proj = (*light_proj) * (*light_view);
            light_ufms[idx].update(renderer, &[view_proj]);
        }
    }

    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderItem<'a> {
        let static_mesh = context
            .resources
            .meshes
            .get(&self.static_mesh)
            .expect("invalid static mesh handle");

        let light_bind_group =
            &shadow_draw_technique::LIGHT_VIEW_PROJS.get().unwrap()[0].bind_group;
        let model_bind_group = &self.model_xform.bind_group;

        let bind_group_refs = vec![light_bind_group, model_bind_group];

        render_job::RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: context.framebuffer_name(self.framebuffer_name.as_str()),
            num_elements: static_mesh.num_elements(self.submesh_idx),
            vertex_buffers: static_mesh.vertex_buffer_slices(self.submesh_idx),
            index_buffer: static_mesh.index_buffer_slice(self.submesh_idx),
            index_format: static_mesh.index_format,
            bind_group: bind_group_refs,
        }
    }
}
