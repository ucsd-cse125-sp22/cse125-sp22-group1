use super::RenderContext;
use super::Technique;
use crate::assets::shaders;
use crate::drawable::util::TransformUniform;

use crate::renderer::*;
use crate::resources::*;

mod shade_direct_technique {
    use crate::drawable::util::TransformUniform;
    use once_cell::sync::OnceCell;

    pub static INV_VIEW_PROJ: OnceCell<TransformUniform<2>> = OnceCell::new();
    pub static LIGHT_VIEW_PROJS: OnceCell<Vec<TransformUniform<1>>> = OnceCell::new();
}

pub struct ShadeDirectTechnique {
    quad_handle: StaticMeshHandle,
    material: material::Material,
}

impl ShadeDirectTechnique {
    const FRAMEBUFFER_NAME: &'static str = "shade_direct_out";
    pub fn new(
        renderer: &Renderer,
        resources: &ResourceManager,
        quad_handle: StaticMeshHandle,
    ) -> Self {
        let material = material::MaterialBuilder::new(renderer, resources, Self::PASS_NAME)
            .framebuffer_texture_resource(0, 0, "geometry_out", 0, false)
            .framebuffer_texture_resource(0, 1, "geometry_out", 1, false)
            .framebuffer_texture_resource(0, 2, "geometry_out", 2, false)
            .framebuffer_texture_resource(0, 3, "shadow_out1", 0, false)
            .produce();

        Self {
            quad_handle,
            material,
        }
    }
}

impl Technique for ShadeDirectTechnique {
    const PASS_NAME: &'static str = "shade_direct";

    fn register(renderer: &mut Renderer) {
        renderer.register_pass(
            Self::PASS_NAME,
            &util::indirect_graphics_nodepth_pass!(
                &shaders::SHADE_DIRECT,
                false,
                [wgpu::TextureFormat::Rgba16Float],
                [Some(wgpu::BlendState::ALPHA_BLENDING)]
            ),
        );

        let res = shade_direct_technique::INV_VIEW_PROJ.set(TransformUniform::new(
            renderer,
            Self::PASS_NAME,
            1,
        ));

        if res.is_err() {
            println!("Re-registering technique but not resetting static uniforms");
        }
    }

    fn update_once(renderer: &Renderer, context: &RenderContext) {
        let light_ufms = shade_direct_technique::LIGHT_VIEW_PROJS.get_or_init(|| {
            let mut light_ufms = vec![];
            for _ in context.light_vps.iter() {
                light_ufms.push(TransformUniform::new(renderer, Self::PASS_NAME, 2));
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

        let view_ufm = shade_direct_technique::INV_VIEW_PROJ.get().unwrap();

        let inv_view = context.view.inverse();
        let inv_proj = context.proj.inverse();
        view_ufm.update(renderer, &[inv_view, inv_proj]);
    }

    fn render_item<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderItem<'a> {
        let static_mesh = context.resources.meshes.get(&self.quad_handle).unwrap();
        let view_bind_group = &shade_direct_technique::INV_VIEW_PROJ
            .get()
            .unwrap()
            .bind_group;
        let light_bind_group =
            &shade_direct_technique::LIGHT_VIEW_PROJS.get().unwrap()[0].bind_group;
        let bind_groups = self
            .material
            .bind_groups(context.iteration)
            .into_iter()
            .chain(std::slice::from_ref(view_bind_group).iter())
            .chain(std::slice::from_ref(light_bind_group).iter())
            .collect();

        render_job::RenderItem::Graphics {
            pass_name: Self::PASS_NAME,
            framebuffer_name: context.framebuffer_name(Self::FRAMEBUFFER_NAME),
            num_elements: static_mesh.num_elements(0),
            vertex_buffers: static_mesh.vertex_buffer_slices(0),
            index_buffer: static_mesh.index_buffer_slice(0),
            index_format: wgpu::IndexFormat::Uint16,
            bind_group: bind_groups,
        }
    }
}
