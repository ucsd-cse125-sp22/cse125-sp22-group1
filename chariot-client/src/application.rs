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
    postprocess: FSQTechnique,
}

impl Application {
    pub fn new(mut renderer: Renderer, game: GameClient) -> Self {
        let mut resources = ResourceManager::new();

        renderer.register_pass(
            "boring",
            &direct_graphics_depth_pass!(include_str!("shader.wgsl")),
        );

        renderer.register_pass(
            "forward",
            &indirect_graphics_depth_pass!(
                include_str!("shader.wgsl"),
                [
                    wgpu::TextureFormat::Rgba16Float,
                    wgpu::TextureFormat::Rgba8Unorm
                ]
            ),
        );

        renderer.register_pass(
            "postprocess",
            &direct_graphics_depth_pass!(include_str!("postprocess.wgsl")),
        );

        let fb_desc = resources.depth_framebuffer(
            "forward_out",
            &renderer,
            &[
                wgpu::TextureFormat::Rgba16Float,
                wgpu::TextureFormat::Rgba8Unorm,
            ],
            Some(wgpu::Color {
                r: 0.517,
                g: 0.780,
                b: 0.980,
                a: 1.0,
            }),
        );

        renderer.register_framebuffer("forward_out", fb_desc);

        let postprocess = FSQTechnique::new(&renderer, &resources, "postprocess");

        let mut world = World::new();

        {
            let chair_import_result = resources.import_gltf(&mut renderer, "models/chair.glb");

            let mut chair = Entity::new();
            chair.set_component(Transform {
                translation: glam::vec3(0.0, 0.5, 0.0),
                rotation: glam::Quat::IDENTITY,
                scale: glam::vec3(1.1995562314987183, 2.2936718463897705, 1.1995562314987183) * 0.2,
            });

            // temporarily commenting this since the new import stuff is in a different branch
            chair.set_component(
                chair_import_result
                    .expect("Failed to import chair")
                    .drawables,
            );

            chair.set_component(Camera {
                orbit_angle: glam::Vec2::ZERO,
                distance: 2.0,
            });

            world.root_mut().add_child(chair);
        }
        {
            let track_import_result = resources.import_gltf(&mut renderer, "models/racetrack.glb");

            let mut track = Entity::new();
            track.set_component(Transform {
                translation: glam::Vec3::ZERO,
                rotation: glam::Quat::IDENTITY,
                scale: glam::vec3(20.0, 20.0, 20.0),
            });

            track.set_component(
                track_import_result
                    .expect("Unable to load racetrack")
                    .drawables,
            );

            world.root_mut().add_child(track);
        }

        Self {
            world: world,
            renderer: renderer,
            resources: resources,
            game,
            pressed_keys: HashSet::new(),
            mouse_pos: PhysicalPosition::<f64> { x: -1.0, y: -1.0 },
            postprocess,
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

        let mut view_local =
            glam::Mat4::look_at_rh(glam::vec3(0.0, 0.0, -2.0), glam::Vec3::ZERO, glam::Vec3::Y);
        let mut view_global = glam::Mat4::IDENTITY;
        dfs_acc(self.world.root_mut(), root_transform.inverse(), |e, acc| {
            let mut cur_model_transform: Transform = e
                .get_component::<Transform>()
                .map_or(Transform::default(), |t| *t);

            cur_model_transform.scale = glam::Vec3::ONE;
            let cur_model = cur_model_transform.to_mat4();

            let acc_model = *acc * cur_model;

            if let Some(camera) = e.get_component::<Camera>() {
                view_local = camera.view_mat4();
                view_global = acc_model;
            }

            acc_model
        });

        let view = view_global.inverse() * view_local;

        let surface_size = self.renderer.surface_size();
        let aspect_ratio = (surface_size.width as f32) / (surface_size.height as f32);
        let proj = glam::Mat4::perspective_rh(f32::to_radians(60.0), aspect_ratio, 0.1, 100.0);
        let proj_view = proj * view;

        let mut render_job = render_job::RenderJob::default();
        dfs_acc(self.world.root_mut(), root_transform, |e, acc| {
            let cur_model = e
                .get_component::<Transform>()
                .unwrap_or(&Transform::default())
                .to_mat4();
            let acc_model = *acc * cur_model;

            if let Some(drawables) = e.get_component::<Vec<StaticMeshDrawable2>>() {
                for drawable in drawables.iter() {
                    drawable.update_xforms(&self.renderer, &proj_view, &acc_model);
                    let render_graph = drawable.render_graph(&self.resources);
                    render_job.merge_graph(render_graph);
                }
            }

            acc_model
        });

        let postprocess_graph = self.postprocess.render_item(&self.resources).to_graph();
        render_job.merge_graph_after("forward", postprocess_graph);

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
