use super::RenderContext;
use super::Technique;
use crate::assets::shaders;
use crate::drawable::util::TransformUniform;
use crate::renderer::*;
use crate::resources::*;

mod geometry_draw_technique {
    use crate::drawable::util::TransformUniform;
    use once_cell::sync::OnceCell;

    pub static VIEW_PROJ: OnceCell<TransformUniform<1>> = OnceCell::new();
}

pub struct GeometryDrawTechnique {
    material: MaterialHandle,
    static_mesh: StaticMeshHandle,
    submesh_idx: usize,
    pub model_xforms: TransformUniform<2>,
}

impl GeometryDrawTechnique {
    pub fn new(
        renderer: &Renderer,
        material: MaterialHandle,
        static_mesh: StaticMeshHandle,
        submesh_idx: usize,
    ) -> Self {
        Self {
            material: material,
            static_mesh: static_mesh,
            submesh_idx: submesh_idx,
            model_xforms: TransformUniform::new(renderer, Self::PASS_NAME, 1),
        }
    }
}

impl Technique for GeometryDrawTechnique {
    const PASS_NAME: &'static str = "geometry";

    fn register(renderer: &mut Renderer) {
        renderer.register_pass(
            Self::PASS_NAME,
            &util::indirect_graphics_depth_pass!(
                &shaders::GEOMETRY,
                true,
                [
                    wgpu::TextureFormat::Rgba16Float,
                    wgpu::TextureFormat::Rgba8Unorm
                ],
                [
                    Some(wgpu::BlendState::ALPHA_BLENDING),
                    Some(wgpu::BlendState::ALPHA_BLENDING)
                ]
            ),
        );

        let res = geometry_draw_technique::VIEW_PROJ.set(TransformUniform::new(
            renderer,
            Self::PASS_NAME,
            0,
        ));

        if res.is_err() {
            println!("Re-registering technique but not resetting static uniforms");
        }
    }

    fn update_once(renderer: &Renderer, context: &RenderContext) {
        let view_ufm = geometry_draw_technique::VIEW_PROJ.get().unwrap();

        let view_proj = context.proj * context.view;
        view_ufm.update(renderer, &[view_proj]);
    }

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

        let view_bind_group = &geometry_draw_technique::VIEW_PROJ.get().unwrap().bind_group;
        let model_bind_group = &self.model_xforms.bind_group;

        let mut bind_group_refs = vec![view_bind_group, model_bind_group];
        bind_group_refs.extend(material.bind_groups(context.iteration));
        render_job::RenderItem::Graphics {
            pass_name: material.pass_name.as_str(),
            framebuffer_name: context.framebuffer_name("geometry_out"),
            num_elements: static_mesh.num_elements(self.submesh_idx),
            vertex_buffers: static_mesh.vertex_buffer_slices(self.submesh_idx),
            index_buffer: static_mesh.index_buffer_slice(self.submesh_idx),
            index_format: static_mesh.index_format,
            bind_group: bind_group_refs,
        }
    }
}

pub struct SurfelGeometryDrawTechnique {
    material: MaterialHandle,
    static_mesh: StaticMeshHandle,
    pub(super) mvp_xform: TransformUniform<3>,
}

// For debugging, not used in prod
impl SurfelGeometryDrawTechnique {
    const FRAMEBUFFER_NAME: &'static str = "geometry_out";

    #[allow(dead_code)]
    pub fn new(
        renderer: &Renderer,
        resources: &ResourceManager,
        material: MaterialHandle,
        static_mesh: StaticMeshHandle,
    ) -> Self {
        let pass_name = &resources
            .materials
            .get(&material)
            .expect("invalid material handle")
            .pass_name;

        Self {
            material: material,
            static_mesh: static_mesh,
            mvp_xform: TransformUniform::new(renderer, pass_name, 0),
        }
    }
}

impl Technique for SurfelGeometryDrawTechnique {
    const PASS_NAME: &'static str = "surfel_geometry";
    fn register(renderer: &mut Renderer) {
        renderer.register_pass(
            Self::PASS_NAME,
            &util::indirect_surfel_pass!(
                &shaders::SURFEL_GEOMETRY,
                [
                    wgpu::TextureFormat::Rgba16Float,
                    wgpu::TextureFormat::Rgba8Unorm
                ]
            ),
        );
    }
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
