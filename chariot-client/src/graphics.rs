use crate::assets;
use crate::assets::models;
use crate::assets::shaders;
use chariot_core::entity_location::EntityLocation;
use chariot_core::player::choices::Chair;
use chariot_core::player::choices::PlayerChoices;
use chariot_core::player::choices::Track;
use chariot_core::player::PlayerID;
use chariot_core::GLOBAL_CONFIG;
use glam::{DVec3, Vec2};
use image::ImageFormat;
use std::f64::consts::PI;

use crate::drawable::particle::ParticleDrawable;
use crate::drawable::technique;
use crate::drawable::technique::CompositeBloomTechnique;
use crate::drawable::technique::DownsampleBloomTechnique;
use crate::drawable::technique::DownsampleTechnique;
use crate::drawable::technique::GeometryDrawTechnique;
use crate::drawable::technique::HBILDebayerTechnique;
use crate::drawable::technique::HBILTechnique;
use crate::drawable::technique::KawaseBlurDownTechnique;
use crate::drawable::technique::KawaseBlurUpTechnique;
use crate::drawable::technique::ShadeDirectTechnique;
use crate::drawable::technique::SimpleFSQTechnique;
use crate::drawable::technique::SkyboxTechnique;
use crate::drawable::technique::Technique;
use crate::drawable::AnimatedUIDrawable;
use crate::drawable::Drawable;
use crate::drawable::RenderContext;
use crate::drawable::StaticMeshDrawable;
use crate::drawable::UIDrawable;
use crate::renderer::*;
use crate::resources::*;
use crate::scenegraph::components::*;
use crate::scenegraph::particle_system::*;
use crate::scenegraph::*;
use crate::ui_state::UIState;
use crate::ui_state::{AnnouncementState, CountdownState};

pub fn register_passes(renderer: &mut Renderer) {
    StaticMeshDrawable::register(renderer);
    UIDrawable::register(renderer);
    ParticleDrawable::register(renderer);

    ShadeDirectTechnique::register(renderer);
    SkyboxTechnique::register(renderer);

    DownsampleTechnique::register(renderer);

    HBILTechnique::register(renderer);
    HBILDebayerTechnique::register(renderer);

    DownsampleBloomTechnique::register(renderer);
    KawaseBlurDownTechnique::register(renderer);
    KawaseBlurUpTechnique::register(renderer);
    CompositeBloomTechnique::register(renderer);

    SimpleFSQTechnique::register(renderer);

    renderer.register_pass(
        "init_probes",
        &util::indirect_surfel_pass!(&shaders::INIT_PROBES, [wgpu::TextureFormat::Rgba16Float]),
    );

    renderer.register_pass(
        "temporal_acc_probes",
        &util::indirect_surfel_pass!(
            &shaders::TEMPORAL_ACC_PROBES,
            [wgpu::TextureFormat::Rgba16Float]
        ),
    );

    renderer.register_pass(
        "geometry_acc_probes",
        &util::indirect_surfel_pass!(
            &shaders::GEOMETRY_ACC_PROBES,
            [wgpu::TextureFormat::Rgba16Float]
        ),
    );
}

fn setup_void() -> World {
    let mut world = World::new();
    world.register::<Camera>();
    world.register::<Vec<StaticMeshDrawable>>();
    world.register::<Bounds>();
    world.register::<Light>();
    world.register::<FlyCamera>();

    ParticleSystem::<0>::register_components(&mut world);
    ParticleSystem::<1>::register_components(&mut world);

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
    pub ui: UIState,
    pub player_num: PlayerID,
    pub player_choices: [Option<PlayerChoices>; 4],
    pub player_entities: [Option<Entity>; 4],
    shade_direct: ShadeDirectTechnique,
    skybox: SkyboxTechnique,
    downsample: DownsampleTechnique,
    hibl: HBILTechnique,
    hibl_debayer: HBILDebayerTechnique,
    downsample_bloom: DownsampleBloomTechnique,
    kawase_blur_down: KawaseBlurDownTechnique,
    kawase_blur_up: KawaseBlurUpTechnique,
    composite_bloom: CompositeBloomTechnique,
    simple_fsq: SimpleFSQTechnique,
    fire_particle_system: ParticleSystem<0>,
    smoke_particle_system: ParticleSystem<1>,
    prev_view: glam::Mat4,
    prev_proj: glam::Mat4,
    iteration: u32,
    camera_entity: Entity,
    pub test_ui: AnimatedUIDrawable,
    pub white_box_tex: TextureHandle,
}

impl GraphicsManager {
    pub fn new(mut renderer: Renderer) -> Self {
        let mut resources = ResourceManager::new();

        register_passes(&mut renderer);

        resources.register_depth_surface_framebuffer(
            "geometry_out",
            &mut renderer,
            &[
                wgpu::TextureFormat::Rgba16Float,
                wgpu::TextureFormat::Rgba8Unorm,
            ],
            Some(wgpu::Color::TRANSPARENT),
            true,
            true,
        );

        let shadow_map_res = winit::dpi::PhysicalSize::<u32>::new(2048, 2048);
        resources.register_depth_framebuffer(
            "shadow_out1",
            &mut renderer,
            shadow_map_res,
            &[],
            None,
            true,
            false,
        );

        resources.register_depth_surface_framebuffer(
            "particles_out",
            &mut renderer,
            &[wgpu::TextureFormat::Rgba8Unorm],
            Some(wgpu::Color::TRANSPARENT),
            true,
            false,
        );

        resources.register_depth_surface_framebuffer(
            "probes_out",
            &mut renderer,
            &[wgpu::TextureFormat::Rgba16Float],
            Some(wgpu::Color::WHITE),
            true,
            true,
        );

        resources.register_surface_framebuffer(
            "shade_direct_out",
            &mut renderer,
            &[wgpu::TextureFormat::Rgba16Float],
            Some(wgpu::Color::TRANSPARENT),
            false,
        );

        let surface_size = renderer.surface_size();
        let downsample2_size =
            winit::dpi::PhysicalSize::<u32>::new(surface_size.width / 2, surface_size.height / 2);

        resources.register_framebuffer(
            "shade_direct_out_0_ds",
            &mut renderer,
            downsample2_size,
            &[wgpu::TextureFormat::Rgba16Float],
            Some(wgpu::Color::TRANSPARENT),
            false,
        );

        resources.register_surface_framebuffer(
            "hbil_out",
            &mut renderer,
            &[wgpu::TextureFormat::Rgba8Unorm],
            Some(wgpu::Color::TRANSPARENT),
            false,
        );

        resources.register_surface_framebuffer(
            "hbil_debayer_out",
            &mut renderer,
            &[wgpu::TextureFormat::Rgba8Unorm],
            Some(wgpu::Color::TRANSPARENT),
            false,
        );

        let downsample4_size =
            winit::dpi::PhysicalSize::<u32>::new(surface_size.width / 4, surface_size.height / 4);
        resources.register_framebuffer(
            "downsample_bloom_out",
            &mut renderer,
            downsample4_size,
            &[wgpu::TextureFormat::Rgba8Unorm],
            Some(wgpu::Color::TRANSPARENT),
            false,
        );

        let downsample8_size =
            winit::dpi::PhysicalSize::<u32>::new(surface_size.width / 8, surface_size.height / 8);
        resources.register_framebuffer(
            "kawase_blur_down_out",
            &mut renderer,
            downsample8_size,
            &[wgpu::TextureFormat::Rgba8Unorm],
            Some(wgpu::Color::TRANSPARENT),
            false,
        );

        resources.register_framebuffer(
            "kawase_blur_up_out",
            &mut renderer,
            downsample4_size,
            &[wgpu::TextureFormat::Rgba8Unorm],
            Some(wgpu::Color::TRANSPARENT),
            false,
        );

        resources.register_surface_framebuffer(
            "composite_bloom_out",
            &mut renderer,
            &[wgpu::TextureFormat::Rgba8Unorm],
            Some(wgpu::Color::TRANSPARENT),
            false,
        );

        /*resources.register_depth_surface_framebuffer(
            "probes_acc_out",
            &mut renderer,
            &[wgpu::TextureFormat::Rgba16Float],
            Some(wgpu::Color::BLACK),
            true,
        );*/

        let quad_handle = resources.create_quad_mesh(&renderer);
        let fire_handle = resources.import_texture_embedded(
            &renderer,
            "sprites/fire",
            assets::sprites::FIRE,
            ImageFormat::Png,
        );
        //let fire_offset = glam::Vec3::Z * -3.0;
        let fire_particle_system = ParticleSystem::new(
            &renderer,
            &mut resources,
            ParticleSystemParams {
                texture_handle: fire_handle,
                mesh_handle: quad_handle,
                pos_range: (-glam::Vec3::ONE * 0.2, glam::Vec3::ONE * 0.2),
                size_range: (glam::vec2(0.9, 1.9), glam::vec2(1.1, 2.1)),
                initial_vel: glam::Vec3::ZERO,
                spawn_rate: 50.0,
                lifetime: 1.0,
                rotation: ParticleRotation::RandomAroundAxis(glam::Vec3::Y),
                gravity: 0.0,
            },
        );

        let smoke_handle = resources.import_texture_embedded(
            &renderer,
            "sprites/smoke",
            assets::sprites::SMOKE,
            ImageFormat::Png,
        );
        let smoke_particle_system = ParticleSystem::new(
            &renderer,
            &mut resources,
            ParticleSystemParams {
                texture_handle: smoke_handle,
                mesh_handle: quad_handle,
                pos_range: (-glam::Vec3::ONE * 0.1, glam::Vec3::ONE * 0.1),
                size_range: (Vec2::ONE, Vec2::ONE * 3.0),
                initial_vel: glam::Vec3::ZERO,
                spawn_rate: 50.0,
                lifetime: 5.0,
                rotation: ParticleRotation::Billboard,
                gravity: -0.2,
            },
        );
        let world = setup_void();
        let shade_direct = ShadeDirectTechnique::new(&renderer, &resources, quad_handle);
        let skybox = SkyboxTechnique::new(&renderer, &resources, quad_handle);
        let downsample =
            DownsampleTechnique::new(&renderer, &resources, "shade_direct_out", 0, quad_handle);
        let hibl = HBILTechnique::new(&renderer, &resources, quad_handle);
        let hibl_debayer = HBILDebayerTechnique::new(&renderer, &resources, quad_handle);
        let downsample_bloom = DownsampleBloomTechnique::new(&renderer, &resources, quad_handle);
        let kawase_blur_down = KawaseBlurDownTechnique::new(&renderer, &resources, quad_handle);
        let kawase_blur_up = KawaseBlurUpTechnique::new(&renderer, &resources, quad_handle);
        let composite_bloom = CompositeBloomTechnique::new(&renderer, &resources, quad_handle);
        let simple_fsq =
            SimpleFSQTechnique::new(&renderer, &resources, "composite_bloom_out", 0, quad_handle);

        let white_box_tex = resources.import_texture_embedded(
            &renderer,
            "box.png",
            assets::ui::WHITE_TEXTURE,
            ImageFormat::Png,
        );

        Self {
            world,
            renderer,
            resources,
            player_choices: Default::default(),
            prev_view: glam::Mat4::IDENTITY,
            prev_proj: glam::Mat4::IDENTITY,
            iteration: 0,
            shade_direct,
            skybox,
            downsample,
            hibl,
            hibl_debayer,
            downsample_bloom,
            kawase_blur_down,
            kawase_blur_up,
            composite_bloom,
            simple_fsq,
            player_entities: [None, None, None, None],
            ui: UIState::None,
            player_num: 4,
            fire_particle_system,
            smoke_particle_system,
            camera_entity: NULL_ENTITY,
            test_ui: AnimatedUIDrawable::new(),
            white_box_tex,
        }
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
                orbit_angle: Vec2::ZERO,
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
        world.register::<FlyCamera>();

        ParticleSystem::<0>::register_components(&mut world);
        ParticleSystem::<1>::register_components(&mut world);

        let world_root = world.root();

        {
            let track_import = self
                .resources
                .import_gltf_file(
                    &mut self.renderer,
                    &format!("{}/{}.glb", GLOBAL_CONFIG.tracks_folder, map.to_string()),
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
                    glam::vec3(-1.0, -0.5, 0.0),
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

    pub fn load_dev_mode(&mut self, map: Track) {
        self.world = self.setup_world(map);
        let world_root = self.world.root();
        let cam = self
            .world
            .builder()
            .attach(world_root)
            .with(Transform {
                translation: glam::vec3(0.0, 5.0, 0.0),
                rotation: glam::Quat::IDENTITY,
                scale: glam::Vec3::ONE * 0.2,
            })
            .with(FlyCamera {
                angle: glam::vec2(std::f32::consts::PI, 0.0),
            })
            .build();
        self.camera_entity = cam;

        let chair_import = self
            .resources
            .import_gltf_slice(&mut self.renderer, models::get_chair_data(Chair::Swivel))
            .expect("Failed to import chair");

        let world_root = self.world.root();
        let chair = self
            .world
            .builder()
            .attach(world_root)
            .with(Transform {
                translation: glam::vec3(0.0, 5.0, 5.0),
                rotation: glam::Quat::IDENTITY,
                scale: glam::Vec3::ONE * 0.2,
            })
            .with(chair_import.drawables)
            .with(chair_import.bounds)
            .build();

        self.player_entities[0] = Some(chair);
    }

    pub fn add_player(&mut self, player_num: PlayerID) {
        let is_self = self.player_num == player_num;
        let choices = self.player_choices[player_num].clone().unwrap_or_default();
        println!("Adding new player: {}, self? {}", player_num, is_self);

        let chair_import = self
            .resources
            .import_gltf_slice(&mut self.renderer, models::get_chair_data(choices.chair))
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
                    orbit_angle: Vec2::ZERO,
                    distance: 3.0,
                },
            );

            self.camera_entity = chair;
        }

        self.player_entities[player_num as usize] = Some(chair);
    }

    pub fn update(&mut self, delta_time: f32) {
        self.fire_particle_system
            .update(&mut self.world, delta_time);
        self.smoke_particle_system
            .update(&mut self.world, delta_time);
    }

    pub fn update_flycam_angle(&mut self, x: f64, y: f64) {
        let screen_sizei = self.renderer.surface_size();
        let screen_size = glam::uvec2(screen_sizei.width, screen_sizei.height).as_vec2();
        let mouse_pos = glam::dvec2(x, y).as_vec2();
        if let Some(camera) = self.world.get_mut::<FlyCamera>(self.camera_entity) {
            let ndc = (mouse_pos / screen_size) * 2.0 - 1.0;
            camera.angle = glam::vec2(
                std::f32::consts::PI - ndc.x * std::f32::consts::PI,
                ndc.y * std::f32::consts::FRAC_PI_2,
            );
        }
    }

    pub fn update_flycam_pos(&mut self, dir: glam::Vec3) {
        let mut new_transform = *self
            .world
            .get::<Transform>(self.camera_entity)
            .unwrap_or(&Transform::default());
        if let Some(camera) = self.world.get_mut::<FlyCamera>(self.camera_entity) {
            let forward = camera.look_dir();
            let right = forward.cross(glam::Vec3::Y).normalize();
            new_transform.translation += 0.3 * (forward * dir.z + right * dir.x);
        }

        if let Some(transform) = self.world.get_mut::<Transform>(self.camera_entity) {
            *transform = new_transform;
        }
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
        let new_player_transform =
            Transform::from_entity_location(&location, player_transform.scale);
        *player_transform = new_player_transform;

        // If we are moving, we might need to rotate the model
        if velocity.length() > 0.0 {
            if let Some(Chair::Beanbag) = self.player_choices[player_num].as_ref().map(|c| c.chair)
            {
                for drawable in self
                    .world
                    .get_mut::<Vec<StaticMeshDrawable>>(player_entity)
                    .unwrap()
                    .iter_mut()
                {
                    // We need an axis and angle of orientation
                    // Axis is what we spin on
                    // Angle is how far we spin
                    // If we are moving towards velocity, we want to spin "downward" towards it
                    // Thus, the axis is the "right-left" of velocity
                    // This is what we get here
                    let axis = velocity.as_vec3().cross(glam::Vec3::Y).normalize();
                    drawable.modifiers.rotation = Some(
                        glam::Quat::from_axis_angle(
                            axis,
                            // For the angle, we want to move the velocity's length
                            // But we could go either + or - that amount
                            // We want to go "towards the ground"
                            // So we figure out if the angle between velocity and the Y axis is + or -
                            // And then go from there!
                            -(axis.angle_between(-glam::Vec3::Y)).signum()
                                * velocity.length() as f32,
                        )
                        .normalize()
                        .mul_quat(drawable.modifiers.rotation.unwrap_or_default()),
                    );
                }
            }
        }

        if player_entity == self.camera_entity {
            if let Some(camera) = self.world.get_mut::<Camera>(self.camera_entity) {
                match self.player_choices[player_num]
                    .as_ref()
                    .unwrap()
                    .clone()
                    .chair
                    .cam()
                {
                    chariot_core::player::choices::CameraType::FaceForwards => {
                        camera.orbit_angle = Vec2::new(0.0, -0.3).lerp(camera.orbit_angle, 0.5);
                    }
                    chariot_core::player::choices::CameraType::FaceVelocity => {
                        if *velocity != DVec3::ZERO {
                            // first we have to compensate for the rotation of the chair model
                            let rotation_angle =
                                location.unit_steer_direction.angle_between(DVec3::X);
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
                };
            }
        }
    }

    pub fn add_fire_to_player(&mut self, player_num: PlayerID, delta_time: f32) {
        let maybe_player_entity = self.player_entities[player_num as usize];
        if maybe_player_entity.is_none() {
            return;
        }

        let player_entity = maybe_player_entity.unwrap();
        let player_transform = *self.world.get::<Transform>(player_entity).unwrap();
        let world_root = self.world.root();

        let fire_rot = glam::Quat::from_axis_angle(glam::Vec3::X, std::f32::consts::FRAC_PI_2);
        let fire_transform = Transform {
            translation: glam::Vec3::Z * -3.0,
            rotation: fire_rot,
            scale: glam::Vec3::ONE,
        };
        self.fire_particle_system.spawn(
            &self.renderer,
            &mut self.world,
            &fire_transform,
            player_entity,
            delta_time,
        );

        self.smoke_particle_system.spawn(
            &self.renderer,
            &mut self.world,
            &player_transform,
            world_root,
            delta_time,
        );
    }

    pub fn render(&mut self) {
        self.update_dynamic_ui();

        let world_root = self.world.root();
        let root_xform = self
            .world
            .get::<Transform>(world_root)
            .expect("Root doesn't have transform component")
            .to_mat4();

        let surface_size = self.renderer.surface_size();
        let aspect_ratio = (surface_size.width as f32) / (surface_size.height as f32);
        let proj = glam::Mat4::perspective_rh(f32::to_radians(60.0), aspect_ratio, 0.1, 1000.0);

        let view = {
            let mut cur_entity = self.camera_entity;

            let local_view = if let Some(camera) = self.world.get::<Camera>(self.camera_entity) {
                camera.view_mat4()
            } else if let Some(camera) = self.world.get::<FlyCamera>(self.camera_entity) {
                camera.view_mat4()
            } else {
                glam::Mat4::IDENTITY
            };

            let mut global_view_inv = glam::Mat4::IDENTITY;
            while cur_entity != NULL_ENTITY {
                let mut cur_model_transform: Transform = self
                    .world
                    .get::<Transform>(cur_entity)
                    .map_or(Transform::default(), |t| *t);
                cur_model_transform.scale = glam::Vec3::ONE;

                let cur_model = cur_model_transform.to_mat4();
                global_view_inv = cur_model * global_view_inv;

                cur_entity = self.world.get::<SceneNode>(cur_entity).unwrap().parent;
            }

            local_view * global_view_inv.inverse()
        };

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

        let light_vps: Vec<(glam::Mat4, glam::Mat4)> = self
            .world
            .storage::<Light>()
            .unwrap_or(&VecStorage::default())
            .iter()
            .map(|l| l.calc_view_proj(&view_bounds))
            .collect();

        let default_translation = Transform::default();
        let render_context = RenderContext {
            resources: &self.resources,
            iteration: self.iteration,
            view,
            proj,
            light_vps,
        };

        StaticMeshDrawable::update_once(&self.renderer, &render_context);
        ParticleDrawable::update_once(&self.renderer, &render_context);
        ShadeDirectTechnique::update_once(&self.renderer, &render_context);
        SkyboxTechnique::update_once(&self.renderer, &render_context);
        HBILTechnique::update_once(&self.renderer, &render_context);

        let mut render_job = render_job::RenderJob::default();
        self.world.dfs_acc(self.world.root(), root_xform, |e, acc| {
            let cur_transform = self
                .world
                .get::<Transform>(e)
                .unwrap_or(&default_translation);
            let cur_model = cur_transform.to_mat4();
            let acc_model = *acc * cur_model;

            if let Some(drawables) = self.world.get::<Vec<StaticMeshDrawable>>(e) {
                for drawable in drawables.iter() {
                    let mut acc_model = acc_model;

                    if drawable.modifiers.absolute_angle {
                        acc_model = *acc
                            * Transform {
                                translation: cur_transform.translation,
                                rotation: glam::Quat::IDENTITY,
                                scale: cur_transform.scale,
                            }
                            .to_mat4();
                    } else if let Some(rotation) = drawable.modifiers.rotation {
                        acc_model = *acc
                            * Transform {
                                translation: cur_transform.translation,
                                rotation,
                                scale: cur_transform.scale,
                            }
                            .to_mat4();
                    }

                    drawable.update_model(&self.renderer, acc_model, view);
                    let render_graph = drawable.render_graph(&render_context);
                    render_job.merge_graph(render_graph);
                }
            }

            if let Some(drawable) = self
                .world
                .get::<Option<ParticleDrawable>>(e)
                .filter(|d| d.is_some())
                .map(|d| d.as_ref().unwrap())
            {
                if let Some(particle_model) =
                    self.fire_particle_system
                        .calc_particle_model(&self.world, e, view)
                {
                    drawable.update_model(&self.renderer, acc_model * particle_model);
                }

                if let Some(particle_model) =
                    self.smoke_particle_system
                        .calc_particle_model(&self.world, e, view)
                {
                    drawable.update_model(&self.renderer, acc_model * particle_model);
                }
            }

            acc_model
        });

        if let Some(drawables) = self.world.storage::<Option<ParticleDrawable>>() {
            for maybe_drawable in drawables.iter() {
                if let Some(drawable) = maybe_drawable {
                    let graph = drawable.render_graph(&render_context);
                    render_job.merge_graph_after(GeometryDrawTechnique::PASS_NAME, graph);
                }
            }
        }

        let skybox_graph = self.skybox.render_item(&render_context).to_graph();
        render_job.merge_graph_after(ParticleDrawable::PASS_NAME, skybox_graph);

        let shade_direct_graph = self.shade_direct.render_item(&render_context).to_graph();
        render_job.merge_graph_after(SkyboxTechnique::PASS_NAME, shade_direct_graph);

        let downsample_graph = self.downsample.render_item(&render_context).to_graph();
        render_job.merge_graph_after(ShadeDirectTechnique::PASS_NAME, downsample_graph);

        let hibl_graph = self.hibl.render_item(&render_context).to_graph();
        render_job.merge_graph_after(DownsampleTechnique::PASS_NAME, hibl_graph);

        let hibl_debayer_graph = self.hibl_debayer.render_item(&render_context).to_graph();
        render_job.merge_graph_after(HBILTechnique::PASS_NAME, hibl_debayer_graph);

        let downsample_bloom_graph = self
            .downsample_bloom
            .render_item(&render_context)
            .to_graph();
        render_job.merge_graph_after(DownsampleTechnique::PASS_NAME, downsample_bloom_graph);

        let kawase_down_graph = self
            .kawase_blur_down
            .render_item(&render_context)
            .to_graph();
        render_job.merge_graph_after(DownsampleBloomTechnique::PASS_NAME, kawase_down_graph);

        let kawase_up_graph = self.kawase_blur_up.render_item(&render_context).to_graph();
        render_job.merge_graph_after(KawaseBlurDownTechnique::PASS_NAME, kawase_up_graph);

        let composite_bloom_graph = self.composite_bloom.render_item(&render_context).to_graph();
        render_job.merge_graph_after(KawaseBlurUpTechnique::PASS_NAME, composite_bloom_graph);

        let fsq_graph = self.simple_fsq.render_item(&render_context).to_graph();
        render_job.merge_graph_after(CompositeBloomTechnique::PASS_NAME, fsq_graph);

        match &self.ui {
            UIState::None => {}
            UIState::ChairacterSelect {
                background,
                chair_select_box,
                chair_description,
                player_chair_images,
            } => {
                let background_graph = background.render_graph(&render_context);
                render_job.merge_graph_after(SimpleFSQTechnique::PASS_NAME, background_graph);

                let chair_select_box_graph = chair_select_box.render_graph(&render_context);
                render_job.merge_graph_after(SimpleFSQTechnique::PASS_NAME, chair_select_box_graph);

                for chair_image in player_chair_images.iter().flatten() {
                    let chair_graph = chair_image.render_graph(&render_context);
                    render_job.merge_graph_after(SimpleFSQTechnique::PASS_NAME, chair_graph);
                }

                let chair_description_box_graph = chair_description.render_graph(&render_context);
                render_job
                    .merge_graph_after(SimpleFSQTechnique::PASS_NAME, chair_description_box_graph);
            }
            UIState::InGameHUD {
                place_position_image,
                game_announcement_title,
                game_announcement_subtitle,
                announcement_state,
                minimap_ui,
                timer_ui,
                lap_ui,
                interaction_ui,
                countdown_ui,
                ..
            } => {
                let position_graph = place_position_image.render_graph(&render_context);
                render_job.merge_graph_after(SimpleFSQTechnique::PASS_NAME, position_graph);

                if let AnnouncementState::None = announcement_state {
                } else {
                    let text_graph = game_announcement_title.render_graph(&render_context);
                    render_job.merge_graph_after(SimpleFSQTechnique::PASS_NAME, text_graph);

                    let text_graph = game_announcement_subtitle.render_graph(&render_context);
                    render_job.merge_graph_after(SimpleFSQTechnique::PASS_NAME, text_graph);
                }
                let minimap_ui_graph = minimap_ui.render_graph(&render_context);
                render_job.merge_graph_after(SimpleFSQTechnique::PASS_NAME, minimap_ui_graph);

                let lap_ui_graph = lap_ui.render_graph(&render_context);
                render_job.merge_graph_after(SimpleFSQTechnique::PASS_NAME, lap_ui_graph);

                let timer_ui_graph = timer_ui.render_graph(&render_context);
                render_job.merge_graph_after(SimpleFSQTechnique::PASS_NAME, timer_ui_graph);

                if let Some(countdown_ui) = countdown_ui {
                    let countdown_ui_graph = countdown_ui.render_graph(&render_context);
                    render_job.merge_graph_after(SimpleFSQTechnique::PASS_NAME, countdown_ui_graph);
                }

                // commenting out now, will merge this in later
                // let interaction_ui_graph = interaction_ui.render_graph(&render_context);
                // render_job.merge_graph_after(SimpleFSQTechnique::PASS_NAME, interaction_ui_graph);
            }
            UIState::MainMenu { background } => {
                let ui_graph = background.render_graph(&render_context);
                render_job.merge_graph_after(SimpleFSQTechnique::PASS_NAME, ui_graph);
            }
            UIState::FinalStandings {
                final_standings_ui,
                player_final_times,
            } => {
                let ui_graph = final_standings_ui.render_graph(&render_context);
                render_job.merge_graph_after(SimpleFSQTechnique::PASS_NAME, ui_graph);

                for player_final_time in player_final_times {
                    let time_graph = final_standings_ui.render_graph(&render_context);
                    render_job.merge_graph_after(SimpleFSQTechnique::PASS_NAME, time_graph);
                }
            }
        }

        self.renderer.render(&render_job);

        self.prev_view = view;
        self.prev_proj = proj;
        self.iteration += 1;
    }
}
