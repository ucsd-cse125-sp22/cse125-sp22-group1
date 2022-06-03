use super::RenderContext;
use super::Technique;
use crate::assets::shaders;
use crate::drawable::util::TransformUniform;
use crate::renderer::render_job::RenderItem;
use crate::renderer::*;
use crate::resources::material::Material;
use crate::resources::material::MaterialBuilder;
use crate::resources::*;

mod shadow_draw_technique {
    use crate::drawable::util::TransformUniform;
    use once_cell::sync::OnceCell;
    pub static LIGHT_VIEW_PROJS: OnceCell<Vec<TransformUniform<2>>> = OnceCell::new();
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
            light_ufms[idx].update(renderer, &[*light_view, *light_proj]);
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

pub struct VSMBlurHorizTechnique {
    quad_handle: StaticMeshHandle,
    material: Material,
    framebuffer_name: String,
}

impl VSMBlurHorizTechnique {
    pub fn new(
        renderer: &Renderer,
        resources: &ResourceManager,
        quad_handle: StaticMeshHandle,
        light_idx: usize,
    ) -> Self {
        let shadow_out_name = format!("shadow_out{}", light_idx);
        let material = MaterialBuilder::new(renderer, resources, Self::PASS_NAME)
            .framebuffer_texture_resource(0, 0, shadow_out_name.as_str(), 0, false)
            .produce();

        Self {
            quad_handle,
            material,
            framebuffer_name: format!("vsm_blur_horiz_out{}", light_idx),
        }
    }
}

impl Technique for VSMBlurHorizTechnique {
    const PASS_NAME: &'static str = "vsm_blur_horiz";
    fn register(renderer: &mut Renderer) {
        renderer.register_pass(
            Self::PASS_NAME,
            &util::indirect_graphics_nodepth_pass!(
                &shaders::VSM_BLUR_HORIZ,
                false,
                [wgpu::TextureFormat::Rg16Float],
                [None]
            ),
        );
    }

    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> RenderItem<'a> {
        let static_mesh = context.resources.meshes.get(&self.quad_handle).unwrap();
        let bind_groups = self.material.bind_groups(context.iteration);

        RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: context.framebuffer_name(self.framebuffer_name.as_str()),
            num_elements: static_mesh.num_elements(0),
            vertex_buffers: static_mesh.vertex_buffer_slices(0),
            index_buffer: static_mesh.index_buffer_slice(0),
            index_format: wgpu::IndexFormat::Uint16,
            bind_group: bind_groups,
        }
    }
}

pub struct VSMBlurVertTechnique {
    quad_handle: StaticMeshHandle,
    material: Material,
    framebuffer_name: String,
}

impl VSMBlurVertTechnique {
    pub fn new(
        renderer: &Renderer,
        resources: &ResourceManager,
        quad_handle: StaticMeshHandle,
        light_idx: usize,
    ) -> Self {
        let vsm_blur_horiz_out = format!("vsm_blur_horiz_out{}", light_idx);
        let material = MaterialBuilder::new(renderer, resources, Self::PASS_NAME)
            .framebuffer_texture_resource(0, 0, vsm_blur_horiz_out.as_str(), 0, false)
            .produce();

        Self {
            quad_handle,
            material,
            framebuffer_name: format!("vsm_blur_vert_out{}", light_idx),
        }
    }
}

impl Technique for VSMBlurVertTechnique {
    const PASS_NAME: &'static str = "vsm_blur_vert";

    fn register(renderer: &mut Renderer) {
        renderer.register_pass(
            Self::PASS_NAME,
            &util::indirect_graphics_nodepth_pass!(
                &shaders::VSM_BLUR_VERT,
                false,
                [wgpu::TextureFormat::Rg16Float],
                [None]
            ),
        );
    }

    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> RenderItem<'a> {
        let static_mesh = context.resources.meshes.get(&self.quad_handle).unwrap();
        let bind_groups = self.material.bind_groups(context.iteration);

        RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: context.framebuffer_name(self.framebuffer_name.as_str()),
            num_elements: static_mesh.num_elements(0),
            vertex_buffers: static_mesh.vertex_buffer_slices(0),
            index_buffer: static_mesh.index_buffer_slice(0),
            index_format: wgpu::IndexFormat::Uint16,
            bind_group: bind_groups,
        }
    }
}
