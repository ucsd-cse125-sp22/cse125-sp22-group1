pub mod technique;

use crate::renderer::*;
use crate::resources::*;
use technique::*;
use wgpu::RenderBundle;

pub struct RenderContext<'a> {
    pub resources: &'a ResourceManager,
    pub iteration: u32,
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
    init_probes: InitProbesTechnique,
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
            geometry_draw: GeometryDrawTechnique::new(
                renderer,
                resources,
                material,
                static_mesh,
                submesh_idx,
            ),
            init_probes: InitProbesTechnique::new(renderer, resources, static_mesh),
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
        let normal_to_global = model.inverse().transpose();
        let inv_view = view.inverse();
        let inv_proj = proj.inverse();
        self.geometry_draw
            .mvp_xform
            .update(renderer, &[model, view_proj, normal_to_global]);
        self.init_probes
            .mvp_xform
            .update(renderer, &[model, normal_to_global, inv_view, inv_proj]);
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

            let view_proj = (*light_proj) * (*light_view);
            self.init_probes.light_xform.update(renderer, &[view_proj]);
        }
    }
}

impl Drawable for StaticMeshDrawable {
    fn render_graph<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderGraph<'a> {
        let mut builder = render_job::RenderGraphBuilder::new();

        let mut shadow_deps = vec![];
        for shadow_draw in self.shadow_draws.iter() {
            let item = shadow_draw.render_item(context);
            let dep = builder.add_root(item);
            shadow_deps.push(dep);
        }

        let geometry_item = self.geometry_draw.render_item(context);
        let geometry_dep = builder.add(geometry_item, &shadow_deps);

        let init_probes_item = self.init_probes.render_item(context);
        builder.add(init_probes_item, &[geometry_dep]);

        builder.build()
    }
}
