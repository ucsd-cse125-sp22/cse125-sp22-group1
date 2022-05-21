use chariot_core::entity_location::EntityLocation;
use chariot_core::player::choices::PlayerChoices;
use chariot_core::player::choices::Track;
use chariot_core::player::PlayerID;
use glam::{DVec3, Vec2};
use std::f64::consts::PI;

use crate::drawable::string::StringDrawable;
use crate::drawable::technique::Technique;
use crate::drawable::*;
use crate::renderer::*;
use crate::resources::*;
use crate::scenegraph::components::*;
use crate::scenegraph::*;

pub fn register_passes(renderer: &mut Renderer) {
    renderer.register_pass(
        "forward",
        &util::indirect_graphics_depth_pass!(
            "shaders/forward.wgsl",
            [
                wgpu::TextureFormat::Rgba16Float,
                wgpu::TextureFormat::Rgba8Unorm
            ]
        ),
    );

    renderer.register_pass("shadow", &util::shadow_pass!("shaders/shadow.wgsl"));

    renderer.register_pass(
        "postprocess",
        &util::direct_graphics_nodepth_pass!("shaders/postprocess.wgsl"),
    );

    renderer.register_pass(
        "ui",
        &util::direct_graphics_nodepth_pass!("shaders/ui.wgsl"),
    );
}

fn setup_void() -> World {
    let mut world = World::new();
    world.register::<Camera>();
    world.register::<Vec<StaticMeshDrawable>>();
    world.register::<Bounds>();
    world.register::<Light>();
    let world_root = world.root();

    {
        let scene_bounds = world.calc_bounds(world.root());
        let _light = world
            .builder()
            .attach(world_root)
            .with(Light::new_directional(
                glam::vec3(-0.5, -1.0, 0.5),
                scene_bounds,
            ))
            .with(Transform::default())
            .build();
    }

    world
}

pub struct GraphicsManager {
    pub world: World,
    pub renderer: Renderer,
    pub resources: ResourceManager,
    pub loading_text: StringDrawable,
    pub place_position_text: StringDrawable,
    pub game_announcement_title: StringDrawable,
    pub game_announcement_subtitle: StringDrawable,
    pub player_num: PlayerID,
    pub player_choices: [Option<PlayerChoices>; 4],
    postprocess: technique::FSQTechnique,
    player_entities: [Option<Entity>; 4],
    camera_entity: Entity,
}

impl GraphicsManager {
    pub fn new(mut renderer: Renderer) -> Self {
        let mut resources = ResourceManager::new();

        register_passes(&mut renderer);

        {
            let fb_desc = resources.depth_surface_framebuffer(
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
        }
        {
            // insanely large shadow map for now
            let shadow_map_res = winit::dpi::PhysicalSize::<u32>::new(2048, 2048);
            let fb_desc =
                resources.depth_framebuffer("shadow_out1", &renderer, shadow_map_res, &[], None);
            renderer.register_framebuffer("shadow_out1", fb_desc);
        }

        let mut loading_text = StringDrawable::new("ArialMT", 28.0, Vec2::new(0.005, 0.047), true);
        loading_text.set(
            "Enter sets your chair to standard
sets your map vote to track
; sets your ready status to true
L sets force_start to true
P tells the server to start the next round",
            &renderer,
            &mut resources,
        );
        let mut place_position_text =
            StringDrawable::new("PressStart2P-Regular", 38.0, Vec2::new(0.905, 0.057), false);
        place_position_text.set("tbd", &renderer, &mut resources);
        let postprocess = technique::FSQTechnique::new(&renderer, &resources, "postprocess");

        let mut game_announcement_title =
            StringDrawable::new("ArialMT", 32.0, Vec2::new(0.50, 0.04), false);
        game_announcement_title.set("NO MORE LEFT TURNS", &renderer, &mut resources);
        let mut game_announcement_subtitle =
            StringDrawable::new("ArialMT", 32.0, Vec2::new(0.50, 0.14), false);
        game_announcement_subtitle.set("activating in 20 seconds", &renderer, &mut resources);

        let world = setup_void();

        Self {
            loading_text,
            place_position_text,
            game_announcement_title,
            game_announcement_subtitle,
            postprocess,
            world,
            renderer,
            resources,
            player_choices: Default::default(),
            player_entities: [None, None, None, None],

            player_num: 4,
            camera_entity: NULL_ENTITY,
        }
    }

    pub fn make_announcement(&mut self, title: &str, subtitle: &str) {
        self.game_announcement_title.center_text = true;
        self.game_announcement_subtitle.center_text = true;
        self.game_announcement_title
            .set(title, &self.renderer, &mut self.resources);
        self.game_announcement_subtitle
            .set(subtitle, &self.renderer, &mut self.resources);
        self.game_announcement_title.should_draw = true;
        self.game_announcement_subtitle.should_draw = true;
    }

    pub fn load_menu(&mut self) {
        println!("Loading main menu!");
    }

    pub fn load_pregame(&mut self) {
        println!("Loading pregame screen!");
        self.world = setup_void();
        let root = self.world.root();

        let _camera = self
            .world
            .builder()
            .attach(root)
            .with(Camera {
                orbit_angle: glam::Vec2::ZERO,
                distance: 3.0,
            })
            .build();
    }

    pub fn setup_world(&mut self, map: Track) -> World {
        let mut world = World::new();
        world.register::<Camera>();
        world.register::<Vec<StaticMeshDrawable>>();
        world.register::<Bounds>();
        world.register::<Light>();
        let world_root = world.root();

        {
            self.loading_text
                .set("Loading racetrack...", &self.renderer, &mut self.resources);
            let track_import = self
                .resources
                .import_gltf(
                    &mut self.renderer,
                    format!("models/{}.glb", map.to_string()),
                )
                .expect("Unable to load racetrack");

            let _track = world
                .builder()
                .attach(world_root)
                .with(Transform::default())
                .with(track_import.drawables)
                .with(track_import.bounds)
                .build();
        }

        {
            let scene_bounds = world.calc_bounds(world.root());
            let _light = world
                .builder()
                .attach(world_root)
                .with(Light::new_directional(
                    glam::vec3(-0.5, -1.0, 0.5),
                    scene_bounds,
                ))
                .with(Transform::default())
                .build();
        }

        world
    }

    pub fn load_map(&mut self, map: Track) {
        self.world = self.setup_world(map);

        [0, 1, 2, 3].map(|player_num| self.add_player(player_num));
    }

    pub fn add_player(&mut self, player_num: PlayerID) {
        let is_self = self.player_num == player_num;
        let choices = self.player_choices[player_num].clone().unwrap_or_default();
        println!("Adding new player: {}, self? {}", player_num, is_self);

        self.loading_text
            .set("Loading chair...", &self.renderer, &mut self.resources);
        let chair_import = self
            .resources
            .import_gltf(
                &mut self.renderer,
                format!("models/{}.glb", choices.chair).to_string(),
            )
            .expect("Failed to import chair");

        let world_root = self.world.root();
        let chair = self
            .world
            .builder()
            .attach(world_root)
            .with(Transform {
                translation: glam::vec3(0.0, -100.0, 0.0),
                rotation: glam::Quat::IDENTITY,
                scale: glam::Vec3::ONE * 0.2,
            })
            .with(chair_import.drawables)
            .with(chair_import.bounds)
            .build();

        // Only follow the new chair around if this is us
        if is_self {
            self.world.insert(
                chair,
                Camera {
                    orbit_angle: glam::Vec2::ZERO,
                    distance: 3.0,
                },
            );

            self.camera_entity = chair;
        }

        self.player_entities[player_num as usize] = Some(chair);
    }

    pub fn update_player_location(
        &mut self,
        location: &EntityLocation,
        velocity: &DVec3,
        player_num: PlayerID,
    ) {
        if self.player_entities[player_num as usize].is_none() {
            self.add_player(player_num);
        }
        let player_entity = self.player_entities[player_num as usize].unwrap();

        let player_transform = self
            .world
            .get_mut::<Transform>(player_entity)
            .expect("Trying to update player location when transform does not exist");
        *player_transform = Transform::from_entity_location(&location, player_transform.scale);

        // if this player is the main player, update the camera too (based on velocity)
        if player_entity == self.camera_entity && *velocity != DVec3::ZERO {
            if let Some(camera) = self.world.get_mut::<Camera>(self.camera_entity) {
                // first we have to compensate for the rotation of the chair model
                let rotation_angle = location.unit_steer_direction.angle_between(DVec3::X);
                // next, we add the angle of the direction of the velocity
                let velocity_angle =
                    DVec3::new(velocity.x, 0.0, velocity.z).angle_between(DVec3::X);

                // there's actually some magic trig cancellations happening here that simplify this calculation
                let mut orbit_yaw = velocity.z.signum() * velocity_angle
                    - location.unit_steer_direction.z.signum() * rotation_angle;

                // if the yaw change would be bigger than PI, wrap back around
                let yaw_difference = orbit_yaw - camera.orbit_angle.x as f64;
                if yaw_difference.abs() > PI {
                    orbit_yaw += yaw_difference.signum() * 2.0 * PI;
                }

                // set the new orbit angle complete with magic pitch for now
                camera.orbit_angle =
                    Vec2::new(orbit_yaw as f32, -0.3).lerp(camera.orbit_angle, 0.5);
            }
        }
    }

    pub fn render(&mut self) {
        //let world_bounds = self.world.root().calc_bounds();
        let world_root = self.world.root();
        let root_xform = self
            .world
            .get::<Transform>(world_root)
            .expect("Root doesn't have transform component")
            .to_mat4();

        // Right now, we're iterating over the scene graph and evaluating all the global transforms twice
        // which is kind of annoying. First to find the camera and get the view matrix and again to actually
        // render everything. Ideally maybe in the future this could be simplified

        let surface_size = self.renderer.surface_size();
        let aspect_ratio = (surface_size.width as f32) / (surface_size.height as f32);
        let proj = glam::Mat4::perspective_rh(f32::to_radians(60.0), aspect_ratio, 0.1, 1000.0);

        let mut view_local =
            glam::Mat4::look_at_rh(glam::vec3(0.0, 0.0, -2.0), glam::Vec3::ZERO, glam::Vec3::Y);
        let mut view_global = glam::Mat4::IDENTITY;
        self.world.dfs_acc(self.world.root(), root_xform, |e, acc| {
            let mut cur_model_transform: Transform = self
                .world
                .get::<Transform>(e)
                .map_or(Transform::default(), |t| *t);

            cur_model_transform.scale = glam::Vec3::ONE;
            let cur_model = cur_model_transform.to_mat4();

            let acc_model = *acc * cur_model;

            if let Some(camera) = self.world.get::<Camera>(e) {
                view_local = camera.view_mat4();
                view_global = acc_model;
            }

            acc_model
        });

        let view = view_local * view_global.inverse();

        let view_bounds = {
            let min_z = 0.01;
            let max_z = 0.993;
            let cam_to_world = (proj * view).inverse();
            let corners = [
                glam::Vec3::new(-1.0, -1.0, min_z),
                glam::Vec3::new(1.0, -1.0, min_z),
                glam::Vec3::new(-1.0, 1.0, min_z),
                glam::Vec3::new(1.0, 1.0, min_z),
                glam::Vec3::new(-1.0, -1.0, max_z),
                glam::Vec3::new(1.0, -1.0, max_z),
                glam::Vec3::new(-1.0, 1.0, max_z),
                glam::Vec3::new(1.0, 1.0, max_z),
            ];

            let world_corners: Vec<glam::Vec3> = corners
                .iter()
                .map(|c| {
                    let world_h = cam_to_world * glam::Vec4::new(c.x, c.y, c.z, 1.0);
                    let world = world_h / world_h.w;
                    glam::Vec3::new(world.x, world.y, world.z)
                })
                .collect();

            let min = world_corners
                .clone()
                .into_iter()
                .reduce(|a, c| a.min(c))
                .unwrap();
            let max = world_corners
                .clone()
                .into_iter()
                .reduce(|a, c| a.max(c))
                .unwrap();
            (min, max)
        };

        let lights: Vec<(glam::Mat4, glam::Mat4)> = self
            .world
            .storage::<Light>()
            .unwrap_or(&VecStorage::default())
            .iter()
            .map(|l| l.calc_view_proj(&view_bounds))
            .collect();

        let mut render_job = render_job::RenderJob::default();
        self.world.dfs_acc(self.world.root(), root_xform, |e, acc| {
            let cur_model = self
                .world
                .get::<Transform>(e)
                .unwrap_or(&Transform::default())
                .to_mat4();
            let acc_model = *acc * cur_model;

            if let Some(drawables) = self.world.get::<Vec<StaticMeshDrawable>>(e) {
                for drawable in drawables.iter() {
                    drawable.update_xforms(&self.renderer, proj, view, acc_model);
                    drawable.update_lights(&self.renderer, acc_model, &lights);
                    let render_graph = drawable.render_graph(&self.resources);
                    render_job.merge_graph(render_graph);
                }
            }

            acc_model
        });

        self.postprocess
            .update_view_data(&self.renderer, view, proj);
        self.postprocess.update_light_data(
            &self.renderer,
            lights.first().unwrap().0,
            lights.first().unwrap().1,
        );
        let postprocess_graph = self.postprocess.render_item(&self.resources).to_graph();
        render_job.merge_graph_after("forward", postprocess_graph);

        if self.loading_text.should_draw {
            let text_graph = self.loading_text.render_graph(&self.resources);
            render_job.merge_graph_after("postprocess", text_graph);
        }
        if self.place_position_text.should_draw {
            let text_graph = self.place_position_text.render_graph(&self.resources);
            render_job.merge_graph_after("postprocess", text_graph);
        }
        if self.game_announcement_title.should_draw {
            let text_graph = self.game_announcement_title.render_graph(&self.resources);
            render_job.merge_graph_after("postprocess", text_graph);
        }
        if self.game_announcement_title.should_draw {
            let text_graph = self.game_announcement_title.render_graph(&self.resources);
            render_job.merge_graph_after("postprocess", text_graph);
        }
        if self.game_announcement_subtitle.should_draw {
            let text_graph = self
                .game_announcement_subtitle
                .render_graph(&self.resources);
            render_job.merge_graph_after("postprocess", text_graph);
        }

        self.renderer.render(&render_job);
    }
}
