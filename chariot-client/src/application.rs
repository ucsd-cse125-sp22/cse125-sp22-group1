use gltf::Texture;
use specs::{Join, WorldExt};
use std::{
    cmp::Eq,
    collections::HashMap,
    default,
    sync::atomic::{AtomicUsize, Ordering},
};
use winit::event::{ElementState, VirtualKeyCode};

use crate::client_events::Watching;
use crate::drawable::*;
use crate::game::GameClient;
use crate::renderer::*;
use crate::resources::*;

pub struct Application {
    pub drawables: Vec<StaticMeshDrawable>,
    pub renderer: Renderer,
    pub resources: ResourceManager,
    pub game: GameClient,
}

impl Watching for Application {
    fn on_key_down(&mut self, key: VirtualKeyCode) {
        self.game.on_key_down(key);
    }
    fn on_key_up(&mut self, key: VirtualKeyCode) {
        self.game.on_key_up(key);
    }

    fn on_mouse_move(&mut self, x: f64, y: f64) {
        self.game.on_mouse_move(x, y);
    }
    fn on_left_mouse(&mut self, x: f64, y: f64, state: ElementState) {
        self.game.on_left_mouse(x, y, state);
    }
    fn on_right_mouse(&mut self, x: f64, y: f64, state: ElementState) {
        self.game.on_right_mouse(x, y, state);
    }
}

impl Application {
    pub fn new(renderer: Renderer, game: GameClient) -> Self {
        Self {
            drawables: Vec::new(),
            renderer: renderer,
            resources: ResourceManager::new(),
            game: game,
        }
    }

    pub fn render(&mut self) {
        let view =
            glam::Mat4::look_at_rh(glam::vec3(0.0, 0.0, -1.0), glam::Vec3::ZERO, glam::Vec3::Y);
        let proj = glam::Mat4::perspective_rh(f32::to_radians(60.0), 1.0, 0.01, 1000.0);
        let proj_view = proj * view;
        let model = glam::Mat4::from_translation(glam::vec3(0.0, -0.5, 0.0));

        let mut render_job = render_job::RenderJob::new();
        for drawable in self.drawables.iter() {
            drawable.update_xforms(&self.renderer, &proj_view, &model);
            let render_item = drawable.render_item(&self.resources);
            render_job.add_item(render_item);
        }

        self.renderer.render(&render_job);
    }

    pub fn update(&mut self) {
        self.game.print_keys();
    }

    // TODO: input handlers
}
