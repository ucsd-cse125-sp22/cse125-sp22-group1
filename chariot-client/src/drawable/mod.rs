pub mod technique;

use crate::renderer::*;
use crate::resources::*;
use technique::*;

/*
 * A drawable just produces a render item every frame.
 */
pub trait Drawable {
    fn render_graph<'a>(&'a self, resources: &'a ResourceManager) -> render_job::RenderGraph<'a>;
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
    forward_draw: ForwardDrawTechnique,
}

impl StaticMeshDrawable {
    pub fn new(
        renderer: &Renderer,
        resources: &ResourceManager,
        material: MaterialHandle,
        static_mesh: StaticMeshHandle,
        submesh_idx: usize,
    ) -> Self {
        let shadow_pass = "shadow";

        let shadow_draws = vec![ShadowDrawTechnique::new(
            renderer,
            static_mesh,
            submesh_idx,
            shadow_pass,
            "shadow_out1",
        )];
        Self {
            shadow_draws,
            forward_draw: ForwardDrawTechnique::new(
                renderer,
                resources,
                material,
                static_mesh,
                submesh_idx,
            ),
        }
    }

    pub fn update_xforms(
        &self,
        renderer: &Renderer,
        proj: glam::Mat4,
        view: glam::Mat4,
        model: glam::Mat4,
    ) {
        let view_proj = proj * view;
        let normal_to_local = (view * model).inverse().transpose();
        self.forward_draw
            .mvp_xform
            .update(renderer, &[model, view_proj, normal_to_local]);
    }

    pub fn update_lights(
        &self,
        renderer: &Renderer,
        model: glam::Mat4,
        light_vps: &[(glam::Mat4, glam::Mat4)],
    ) {
        for (idx, (light_view, light_proj)) in light_vps.iter().enumerate() {
            let mvp = (*light_proj) * (*light_view) * model;
            self.shadow_draws[idx].mvp_xform.update(renderer, &[mvp]);
        }
    }
}

impl Drawable for StaticMeshDrawable {
    fn render_graph<'a>(&'a self, resources: &'a ResourceManager) -> render_job::RenderGraph<'a> {
        let mut builder = render_job::RenderGraphBuilder::new();

        let mut shadow_deps = vec![];
        for shadow_draw in self.shadow_draws.iter() {
            let item = shadow_draw.render_item(resources);
            let dep = builder.add_root(item);
            shadow_deps.push(dep);
        }

        let forward_item = self.forward_draw.render_item(resources);
        builder.add(forward_item, &shadow_deps);

        builder.build()
    }
}

pub struct UIDrawable {
    pub layers: Vec<UILayerTechnique>,
}

impl Drawable for UIDrawable {
    fn render_graph<'a>(&'a self, resources: &'a ResourceManager) -> render_job::RenderGraph<'a> {
        let mut builder = render_job::RenderGraphBuilder::new();

        if !self.layers.is_empty() {
            let mut last_dep =
                builder.add_root(self.layers.first().unwrap().render_item(resources));
            for layer in self.layers.iter().skip(1) {
                last_dep = builder.add(layer.render_item(resources), &[last_dep]);
            }
        }

        builder.build()
    }
}
