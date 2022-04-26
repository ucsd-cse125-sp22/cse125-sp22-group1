use std::collections::HashSet;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, VirtualKeyCode};

use crate::drawable::*;
use crate::game::GameClient;
use crate::renderer::*;
use crate::resources::*;
use crate::scenegraph::*;

use chariot_core::player_inputs::{EngineStatus, InputEvent, RotationStatus};

pub struct Application {
    pub world: World,
    pub renderer: Renderer,
    pub resources: ResourceManager,
    pub game: GameClient,
    pub pressed_keys: HashSet<VirtualKeyCode>,
    mouse_pos: PhysicalPosition<f64>,
}

impl Application {
    pub fn new(mut renderer: Renderer, game: GameClient) -> Self {
        renderer.register_pass(
            "boring",
            &direct_graphics_depth_pass!(include_str!("shader.wgsl"), wgpu::IndexFormat::Uint16),
        );

        renderer.register_pass(
            "forward",
            &indirect_graphics_depth_pass!(
                include_str!("shader.wgsl"),
                wgpu::IndexFormat::Uint16,
                [wgpu::TextureFormat::Rgba16Float]
            ),
        );

        renderer.register_pass(
            "postprocess",
            &direct_graphics_nodepth_pass!(
                include_str!("postprocess.wgsl"),
                wgpu::IndexFormat::Uint16
            ),
        );

        let (depth_tex, color_tex, fb_desc) =
            depth_color_framebuffer(&renderer, wgpu::TextureFormat::Rgba16Float);
        renderer.register_framebuffer("forward_out", fb_desc, [depth_tex, color_tex]);

        let mut resources = ResourceManager::new();

        let import_result = resources.import_gltf(&mut renderer, "models/DamagedHelmet.glb");

        let mut world = World::new();
        let mut helmet = Entity::new();
        helmet.set_component(Transform {
            translation: glam::Vec3::ZERO,
            rotation: glam::Quat::from_axis_angle(glam::Vec3::X, f32::to_radians(90.0)),
            scale: glam::vec3(0.3, 0.3, 0.3),
        });

        helmet.set_component(import_result.expect("Failed to import model").drawables);

        helmet.set_component(Camera {
            orbit_angle: glam::Vec2::ZERO,
            distance: 2.0,
        });

        world.root_mut().add_child(helmet);

        Self {
            world: world,
            renderer: renderer,
            resources: resources,
            game,
            pressed_keys: HashSet::new(),
            mouse_pos: PhysicalPosition::<f64> { x: -1.0, y: -1.0 },
        }
    }

    pub fn render(&mut self) {
        let root_transform = self
            .world
            .root()
            .get_component::<Transform>()
            .unwrap_or(&Transform::default())
            .to_mat4();

        // Right now, we're iterating over the scene graph and evaluating all the global transforms once
        // which is kind of annoying. First to find the camera and get the view matrix and again to actually
        // render everything. Ideally maybe in the future this could be simplified

        let mut view_inv_local =
            glam::Mat4::look_at_rh(glam::vec3(0.0, 0.0, -2.0), glam::Vec3::ZERO, glam::Vec3::Y);
        let mut view_global = glam::Mat4::IDENTITY;
        dfs_acc(self.world.root_mut(), root_transform, |e, acc| {
            if let Some(camera) = e.get_component::<Camera>() {
                view_inv_local = camera.view_mat4();
                view_global = *acc;
            }

            let cur_model = e
                .get_component::<Transform>()
                .unwrap_or(&Transform::default())
                .to_mat4();

            let acc_model = *acc * cur_model;

            acc_model
        });

        let view = view_inv_local * view_global.inverse();

        let proj = glam::Mat4::perspective_rh(f32::to_radians(60.0), 1.0, 0.1, 100.0);
        let proj_view = proj * view;

        let mut render_job = render_job::RenderJob::default();
        dfs_acc(self.world.root_mut(), root_transform, |e, acc| {
            let cur_model = e
                .get_component::<Transform>()
                .unwrap_or(&Transform::default())
                .to_mat4();
            let acc_model = *acc * cur_model;

            if let Some(drawables) = e.get_component::<Vec<StaticMeshDrawable>>() {
                for drawable in drawables.iter() {
                    drawable.update_xforms(&self.renderer, &proj_view, &acc_model);
                    let render_graph = drawable.render_graph(&self.resources);
                    render_job.merge_graph(render_graph);
                }
            }

            acc_model
        });

        self.renderer.render(&render_job);
    }

    pub fn update(&mut self) {
        let surface_size = self.renderer.surface_size();
        let surface_size = glam::Vec2::new(surface_size.width as f32, surface_size.height as f32);
        let mouse_pos = glam::Vec2::new(self.mouse_pos.x as f32, self.mouse_pos.y as f32);

        let rot_range = glam::Vec2::new(std::f32::consts::PI, std::f32::consts::FRAC_PI_2);

        dfs_mut(self.world.root_mut(), &|e| {
            if let Some(camera) = e.get_component::<Camera>() {
                let norm_orbit_angle = (mouse_pos / surface_size) * 2.0 - 1.0;
                let orbit_angle = norm_orbit_angle * rot_range;
                let new_camera = Camera {
                    orbit_angle,
                    ..*camera
                };
                e.set_component(new_camera);
            }
        });

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

    fn invert_event(&self, event: Option<InputEvent>) -> Option<InputEvent> {
        Some(match event {
            Some(InputEvent::Engine(EngineStatus::Accelerating)) => {
                InputEvent::Engine(EngineStatus::Neutral)
            }
            Some(InputEvent::Engine(EngineStatus::Braking)) => {
                InputEvent::Engine(EngineStatus::Neutral)
            }
            Some(InputEvent::Rotation(RotationStatus::InSpinClockwise)) => {
                InputEvent::Rotation(RotationStatus::NotInSpin)
            }
            Some(InputEvent::Rotation(RotationStatus::InSpinCounterclockwise)) => {
                InputEvent::Rotation(RotationStatus::NotInSpin)
            }
            _ => return None,
        })
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
            self.game.send_input_event(event);
        };
    }

    pub fn on_key_up(&mut self, key: VirtualKeyCode) {
        println!("Key up [{:?}]!", key);
        self.pressed_keys.remove(&key);

        if let Some(event) = self.invert_event(self.get_input_event(key)) {
            self.game.send_input_event(event);
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
