pub mod particle;
pub mod technique;
pub mod util;

use crate::renderer::*;
use crate::resources::*;
use crate::scenegraph::components::Modifiers;
use technique::ui_layer::UILayerTechnique;
use technique::*;

pub struct RenderContext<'a> {
    pub resources: &'a ResourceManager,
    pub iteration: u32,
    pub view: glam::Mat4,
    pub proj: glam::Mat4,
    pub light_vps: Vec<(glam::Mat4, glam::Mat4)>,
}

impl<'a> RenderContext<'a> {
    pub fn framebuffer_name(&self, name: &str) -> String {
        self.resources
            .framebuffer_name(name, self.iteration % 2 == 1)
    }
}

/*
 * A drawable just produces a render item every frame.
 */
pub trait Drawable {
    fn register(renderer: &mut Renderer);
    fn update_once(_: &Renderer, _: &RenderContext) {}
    fn render_graph<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderGraph<'a>;
}

/*
 * A StaticMeshDrawable produces render items for a single static mesh
 * (or more specifically, a single submesh of a static mesh - weird naming, I know)
 *
 * xform contains the model matrix as well as the view * proj matrix although usually
 * by xform people mean just the model matrix.
 */
pub struct StaticMeshDrawable {
    shadow_draws: Vec<ShadowDrawTechnique>,
    geometry_draw: GeometryDrawTechnique,
    pub modifiers: Modifiers,
}

impl StaticMeshDrawable {
    pub fn new(
        renderer: &Renderer,
        material: MaterialHandle,
        static_mesh: StaticMeshHandle,
        submesh_idx: usize,
    ) -> Self {
        let shadow_draws = vec![ShadowDrawTechnique::new(
            renderer,
            static_mesh,
            submesh_idx,
            "shadow_out1",
        )];
        Self {
            shadow_draws,
            geometry_draw: GeometryDrawTechnique::new(renderer, material, static_mesh, submesh_idx),
            modifiers: Default::default(),
        }
    }

    pub fn update_model(&self, renderer: &Renderer, model: glam::Mat4, view: glam::Mat4) {
        for shadow_draw in self.shadow_draws.iter() {
            shadow_draw.model_xform.update(renderer, &[model]);
        }

        let normal_to_global = (view * model).inverse().transpose();
        self.geometry_draw
            .model_xforms
            .update(renderer, &[model, normal_to_global]);
    }
}

impl Drawable for StaticMeshDrawable {
    fn register(renderer: &mut Renderer) {
        ShadowDrawTechnique::register(renderer);
        GeometryDrawTechnique::register(renderer);
    }

    fn update_once(renderer: &Renderer, context: &RenderContext) {
        ShadowDrawTechnique::update_once(renderer, context);
        GeometryDrawTechnique::update_once(renderer, context);
    }

    fn render_graph<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderGraph<'a> {
        let mut builder = render_job::RenderGraphBuilder::new();

        let mut shadow_deps = vec![];
        for shadow_draw in self.shadow_draws.iter() {
            let item = shadow_draw.render_item(context);
            let dep = builder.add_root(item);
            shadow_deps.push(dep);
        }

        let geometry_item = self.geometry_draw.render_item(context);
        builder.add(geometry_item, &shadow_deps);

        builder.build()
    }
}

pub struct UIDrawable {
    pub layers: Vec<UILayerTechnique>,
}

impl Drawable for UIDrawable {
    fn register(renderer: &mut Renderer) {
        UILayerTechnique::register(renderer);
    }

    fn render_graph<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderGraph<'a> {
        let mut builder = render_job::RenderGraphBuilder::new();

        if !self.layers.is_empty() {
            let mut last_dep = builder.add_root(self.layers.first().unwrap().render_item(context));
            for layer in self.layers.iter().skip(1) {
                last_dep = builder.add(layer.render_item(context), &[last_dep]);
            }
        }

        builder.build()
    }
}
