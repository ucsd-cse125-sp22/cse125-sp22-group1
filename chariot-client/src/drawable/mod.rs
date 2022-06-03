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
    // i am not changing all the cases just to get a single compiler warning to go away :)
    #[allow(dead_code)]
    ColorAnimation {
        start_color: [f32; 4],
        end_color: [f32; 4],
        start_time: Instant,
        duration: Duration,
    }, // TODO?
}

pub struct AnimatedUIDrawable {
    // [(Layer, PositionAnimation?, SizeAnimation?, ColorAnimation?)]
    pub layers: Vec<(
        UILayerTechnique,
        Option<UIAnimation>,
        Option<UIAnimation>,
        Option<UIAnimation>,
    )>,
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
        self.layers.push((ui, None, None, None));
    }

    pub fn pos_to(&mut self, index: usize, end_pos: glam::Vec2, duration: Duration) {
        if let Some((ui, pos_animation, _, _)) = self.layers.get_mut(index) {
            *pos_animation = Some(UIAnimation::PositionAnimation {
                start_pos: ui.pos,
                end_pos,
                start_time: Instant::now(),
                duration,
            });
        }
    }

    pub fn size_to(&mut self, index: usize, end_size: glam::Vec2, duration: Duration) {
        if let Some((ui, _, size_animation, _)) = self.layers.get_mut(index) {
            *size_animation = Some(UIAnimation::SizeAnimation {
                start_size: ui.size,
                end_size,
                start_time: Instant::now(),
                duration,
            });
        }
    }

    pub fn _col_to(&mut self, index: usize, end_color: [f32; 4], duration: Duration) {
        if let Some((ui, _, _, color_animation)) = self.layers.get_mut(index) {
            *color_animation = Some(UIAnimation::ColorAnimation {
                start_color: ui.tint_color,
                end_color,
                start_time: Instant::now(),
                duration,
            });
        }
    }

    pub fn _animate(&mut self, index: usize, anim_vec: Vec<UIAnimation>) {
        if let Some((_, pos_animation, size_animation, color_animation)) =
            self.layers.get_mut(index)
        {
            for anim_data in anim_vec {
                match anim_data {
                    UIAnimation::PositionAnimation { .. } => {
                        *pos_animation = Some(anim_data);
                    }
                    UIAnimation::SizeAnimation { .. } => {
                        *size_animation = Some(anim_data);
                    }
                    UIAnimation::ColorAnimation { .. } => {
                        *color_animation = Some(anim_data);
                    }
                }
            }
        }
    }

    pub fn update(&mut self, renderer: &mut Renderer) {
        let now = Instant::now();
        for (ui, pos_animation, size_animation, color_animation) in self.layers.iter_mut() {
            if let Some(UIAnimation::PositionAnimation {
                start_pos,
                end_pos,
                start_time,
                duration,
            }) = *pos_animation
            {
                if start_time + duration > now {
                    let progress =
                        (now - start_time).as_millis() as f32 / (duration.as_millis() as f32);
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
                if start_time + duration > now {
                    let progress =
                        (now - start_time).as_millis() as f32 / (duration.as_millis() as f32);
                    let change = end_size - start_size;
                    ui.update_size(renderer, change * progress + start_size);
                } else {
                    ui.update_size(renderer, end_size);
                    *size_animation = None;
                }
            }
            if let Some(UIAnimation::ColorAnimation {
                start_color,
                end_color,
                start_time,
                duration,
            }) = *color_animation
            {
                if start_time + duration > now {
                    let progress =
                        (now - start_time).as_millis() as f32 / (duration.as_millis() as f32);
                    let mut new_color: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
                    [0, 1, 2, 3].map(|i| {
                        new_color[i] = (end_color[i] - start_color[i]) * progress + start_color[i]
                    });
                    ui.update_color(renderer, new_color);
                } else {
                    ui.update_color(renderer, end_color);
                    *color_animation = None;
                }
            }
        }

        self.last_update = now;
    }
}

impl Drawable for AnimatedUIDrawable {
    fn register(renderer: &mut Renderer) {
        UILayerTechnique::register(renderer);
    }

    fn render_graph<'a>(&'a self, context: &RenderContext<'a>) -> render_job::RenderGraph<'a> {
        let mut builder = render_job::RenderGraphBuilder::new();

        if !self.layers.is_empty() {
            let mut last_dep =
                builder.add_root(self.layers.first().unwrap().0.render_item(context));
            for (layer, _, _, _) in self.layers.iter().skip(1) {
                last_dep = builder.add(layer.render_item(context), &[last_dep]);
            }
        }

        builder.build()
    }
}
