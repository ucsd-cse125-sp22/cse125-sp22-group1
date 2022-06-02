pub mod particle;
pub mod technique;

use crate::renderer::*;
use crate::resources::*;
use crate::scenegraph::components::Modifiers;
use pathfinder_geometry::util::lerp;
use std::time::{Duration, Instant};
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
    pub modifiers: Modifiers,
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
            modifiers: Default::default(),
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

pub enum UIAnimation {
    PositionAnimation {
        start_pos: glam::Vec2,
        end_pos: glam::Vec2,
        start_time: Instant,
        duration: Duration,
    },
    SizeAnimation {
        start_size: glam::Vec2,
        end_size: glam::Vec2,
        start_time: Instant,
        duration: Duration,
    },
}

pub struct AnimatedUIDrawable {
    // [(Layer, PositionAnimation?, SizeAnimation?)]
    pub layers: Vec<(UILayerTechnique, Option<UIAnimation>, Option<UIAnimation>)>,
    last_update: Instant,
}

impl AnimatedUIDrawable {
    pub fn new() -> AnimatedUIDrawable {
        Self {
            layers: vec![],
            last_update: Instant::now(),
        }
    }

    pub fn push(&mut self, ui: UILayerTechnique) {
        let p = ui.pos.clone();
        let s = ui.size.clone();
        self.layers.push((ui, None, None));
    }

    pub fn animate(
        &mut self,
        index: usize,
        end_pos: Option<glam::Vec2>,
        end_size: Option<glam::Vec2>,
        duration: Duration,
    ) {
        if let Some((ui, pos_animation, size_animation)) = self.layers.get_mut(index) {
            if let Some(pos) = end_pos {
                *pos_animation = Some(UIAnimation::PositionAnimation {
                    start_pos: ui.pos,
                    end_pos: pos,
                    start_time: Instant::now(),
                    duration,
                });
            }

            if let Some(size) = end_size {
                *size_animation = Some(UIAnimation::SizeAnimation {
                    start_size: ui.size,
                    end_size: size,
                    start_time: Instant::now(),
                    duration,
                });
            }
        }
    }

    pub fn update(&mut self, renderer: &mut Renderer) {
        // TODO: real steps, not hard coded
        let timestep = (self.last_update - Instant::now()).as_millis() as u64;

        for (ui, pos_animation, size_animation) in self.layers.iter_mut() {
            if let Some(UIAnimation::PositionAnimation {
                start_pos,
                end_pos,
                start_time,
                duration,
            }) = *pos_animation
            {
                if start_time + duration > Instant::now() {
                    let progress = (Instant::now() - start_time).as_millis() as f32
                        / (duration.as_millis() as f32);
                    let change = end_pos - start_pos;
                    ui.update_pos(renderer, change * progress + start_pos);
                } else {
                    ui.update_pos(renderer, end_pos);
                    *pos_animation = None;
                }
            }
            if let Some(UIAnimation::SizeAnimation {
                start_size,
                end_size,
                start_time,
                duration,
            }) = *size_animation
            {
                if start_time + duration > Instant::now() {
                    let progress = (Instant::now() - start_time).as_millis() as f32
                        / (duration.as_millis() as f32);
                    let change = end_size - start_size;
                    ui.update_size(renderer, change * progress + start_size);
                } else {
                    ui.update_size(renderer, end_size);
                    *size_animation = None;
                }
            }
        }

        self.last_update = Instant::now();
    }
}

impl Drawable for AnimatedUIDrawable {
    fn render_graph<'a>(&'a self, resources: &'a ResourceManager) -> render_job::RenderGraph<'a> {
        let mut builder = render_job::RenderGraphBuilder::new();

        if !self.layers.is_empty() {
            let mut last_dep =
                builder.add_root(self.layers.first().unwrap().0.render_item(resources));
            for (layer, _, _) in self.layers.iter().skip(1) {
                last_dep = builder.add(layer.render_item(resources), &[last_dep]);
            }
        }

        builder.build()
    }
}
