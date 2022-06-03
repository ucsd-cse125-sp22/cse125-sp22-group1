pub mod particle;
pub mod technique;
pub mod util;

use crate::renderer::*;
use crate::resources::*;
use crate::scenegraph::components::Modifiers;
use std::time::{Duration, Instant};
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
    ColorAnimation {
        start_color: todo!(),
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
    fn register(renderer: &mut Renderer) {
        UILayerTechnique::register(renderer);
    }

    fn render_graph<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderGraph<'a> {
        let mut builder = render_job::RenderGraphBuilder::new();

        if !self.layers.is_empty() {
            let mut last_dep = builder.add_root(self.layers.first().unwrap().render_item(context));
            for (layer, _, _) in self.layers.iter().skip(1) {
                last_dep = builder.add(layer.render_item(context), &[last_dep]);
            }
        }

        builder.build()
    }
}
