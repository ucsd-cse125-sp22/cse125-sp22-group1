use std::collections::HashSet;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, VirtualKeyCode};

use crate::drawable::*;
use crate::game::GameClient;
use crate::renderer::*;
use crate::resources::*;
use chariot_core::player_inputs::{EngineStatus, InputEvent, RotationStatus};

pub struct Application {
    pub drawables: Vec<StaticMeshDrawable>,
    pub renderer: Renderer,
    pub resources: ResourceManager,
    pub game: GameClient,
    pub pressed_keys: HashSet<VirtualKeyCode>,
    mouse_pos: PhysicalPosition<f64>,
}

impl Application {
    pub fn new(renderer: Renderer, game: GameClient) -> Self {
        Self {
            drawables: Vec::new(),
            renderer,
            resources: ResourceManager::new(),
            game,
            pressed_keys: HashSet::new(),
            mouse_pos: PhysicalPosition::<f64> { x: -1.0, y: -1.0 },
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
        //self.print_keys();

        self.game.sync_incoming();
    }

    // Input configuration
    fn get_input_event(&self, key: VirtualKeyCode) -> Option<InputEvent> {
        match key {
            // Forwards
            VirtualKeyCode::W => Some(InputEvent::Engine(EngineStatus::Accelerating)),
            // Backwards
            VirtualKeyCode::S => Some(InputEvent::Engine(EngineStatus::Braking)),
            // Left
            VirtualKeyCode::A => Some(InputEvent::Rotation(RotationStatus::InSpinCounterclockwise)),
            // Right
            VirtualKeyCode::D => Some(InputEvent::Rotation(RotationStatus::InSpinClockwise)),
            // Right
            _ => None,
        }
    }

    // Input Handlers
    pub fn on_key_down(&mut self, key: VirtualKeyCode) {
        // winit sends duplicate keydown events, so we will just make sure we don't already have this processed
        if self.pressed_keys.contains(&key) {
            return;
        };

        println!("Key down [{:?}]!", key);
        self.pressed_keys.insert(key);

        if let Some(event) = self.get_input_event(key) {
            self.game.send_input_event(event, true);
        };
    }

    pub fn on_key_up(&mut self, key: VirtualKeyCode) {
        println!("Key up [{:?}]!", key);
        self.pressed_keys.remove(&key);

        if let Some(event) = self.get_input_event(key) {
            self.game.send_input_event(event, false);
        };
    }

    pub fn on_mouse_move(&mut self, x: f64, y: f64) {
        self.mouse_pos.x = x;
        self.mouse_pos.y = y;
        //println!("Mouse moved! ({}, {})", x, y);
    }

    pub fn on_left_mouse(&mut self, state: ElementState) {
        let x = self.mouse_pos.x;
        let y = self.mouse_pos.y;

        if let ElementState::Released = state {
            println!("Mouse clicked @ ({}, {})!", x, y);
        }
    }

    pub fn on_right_mouse(&mut self, state: ElementState) {
        let x = self.mouse_pos.x;
        let y = self.mouse_pos.y;

        if let ElementState::Released = state {
            println!("Mouse right clicked @ ({}, {})!", x, y);
        }
    }

    pub fn print_keys(&self) {
        println!("Pressed keys: {:?}", self.pressed_keys)
    }
}
