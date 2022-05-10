use chariot_core::entity_location::EntityLocation;

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
        &util::direct_graphics_depth_pass!("shaders/postprocess.wgsl"),
    );
}

fn setup_world(resources: &mut ResourceManager, renderer: &mut Renderer) -> (World, Entity) {
    let mut world = World::new();
    world.register::<Camera>();
    world.register::<Vec<StaticMeshDrawable>>();
    world.register::<Bounds>();
    world.register::<Light>();
    let world_root = world.root();
    let chair = {
        let chair_import_result = resources.import_gltf(renderer, "models/chair.glb");
        let chair_import = chair_import_result.expect("Failed to import chair");

        world
            .builder()
            .attach(world_root)
            .with(Transform {
                translation: glam::vec3(0.0, 0.5, 0.0),
                rotation: glam::Quat::IDENTITY,
                scale: glam::vec3(1.1995562314987183, 2.2936718463897705, 1.1995562314987183) * 0.2,
            })
            .with(chair_import.drawables)
            .with(chair_import.bounds)
            .build()
    };
    {
        let track_import_result = resources.import_gltf(renderer, "models/baked.glb");
        let track_import = track_import_result.expect("Unable to load racetrack");

        let track = world
            .builder()
            .attach(world_root)
            .with(Transform {
                translation: glam::Vec3::ZERO,
                rotation: glam::Quat::IDENTITY,
                scale: glam::vec3(20.0, 20.0, 20.0),
            })
            .with(track_import.drawables)
            .with(track_import.bounds)
            .build();
    }

    {
        let scene_bounds = world.calc_bounds(world.root());
        let light = world
            .builder()
            .attach(world_root)
            .with(Light::new_directional(
                glam::vec3(-0.5, -1.0, 0.5),
                scene_bounds,
            ))
            .with(Transform::default())
            .build();
    }

    (world, chair)
}

pub struct GraphicsManager {
    pub world: World,
    pub renderer: Renderer,
    pub resources: ResourceManager,

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
            let shadow_map_res = winit::dpi::PhysicalSize::<u32>::new(8192, 8192);
            let fb_desc =
                resources.depth_framebuffer("shadow_out1", &renderer, shadow_map_res, &[], None);
            renderer.register_framebuffer("shadow_out1", fb_desc);
        }

        let postprocess = technique::FSQTechnique::new(&renderer, &resources, "postprocess");

        let (world, chair) = setup_world(&mut resources, &mut renderer);

        Self {
            world: world,
            renderer: renderer,
            resources: resources,
            postprocess: postprocess,
            player_entities: [Some(chair), None, None, None],
            camera_entity: NULL_ENTITY,
        }
    }

    pub fn add_player(&mut self, player_num: u8, is_self: bool) {
        let chair_import_result = self
            .resources
            .import_gltf(&mut self.renderer, "models/chair.glb");
        let chair_import = chair_import_result.expect("Failed to import chair");

        let world_root = self.world.root();
        let chair = self
            .world
            .builder()
            .attach(world_root)
            .with(Transform {
                translation: glam::vec3(0.0, 0.5, 0.0),
                rotation: glam::Quat::IDENTITY,
                scale: glam::vec3(1.1995562314987183, 2.2936718463897705, 1.1995562314987183) * 0.2,
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
                    distance: 2.0,
                },
            );

            self.camera_entity = chair;
        }

        self.player_entities[player_num as usize] = Some(chair);

        println!("Adding new player: {}, self? {}", player_num, is_self);
    }

    fn EntityLocation_to_Transform(location: &EntityLocation) -> Transform {
        let rotation_1 = glam::Quat::from_rotation_arc(
            glam::Vec3::X,
            location.unit_steer_direction.normalize().as_vec3(),
        );
        let rotation_2 = glam::Quat::from_rotation_arc(
            glam::Vec3::Y,
            location.unit_upward_direction.normalize().as_vec3(),
        );

        return Transform {
            translation: location.position.as_vec3() + glam::Vec3::new(0.0, 1.0, 0.0),
            rotation: rotation_1.mul_quat(rotation_2),
            // only works for chairs! do something more robust for other entities later
            scale: glam::vec3(1.1995562314987183, 2.2936718463897705, 1.1995562314987183) * 0.2,
        };
    }

    pub fn update_player_location(&mut self, location: &EntityLocation, player_num: u8) {
        if self.player_entities[player_num as usize].is_none() {
            self.add_player(player_num, false);
        }
        let player_entity = self.player_entities[player_num as usize].unwrap();

        println!("new location for #{}: {}", player_num, location.position);
        println!(
            "new steer direction for #{}: {}",
            player_num, location.unit_steer_direction
        );
        println!(
            "new upward direction for #{}: {}",
            player_num, location.unit_upward_direction
        );
        let player_transform = self
            .world
            .get_mut::<Transform>(player_entity)
            .expect("Trying to update player location when transform does not exist");
        *player_transform = GraphicsManager::EntityLocation_to_Transform(&location);
    }

    pub fn render(&mut self) {
        //let world_bounds = self.world.root().calc_bounds();
        let world_root = self.world.root();
        let root_xform = self
            .world
            .get::<Transform>(world_root)
            .expect("Root doesn't have transform component")
            .to_mat4();

        // Right now, we're iterating over the scene graph and evaluating all the global transforms once
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

        self.renderer.render(&render_job);
    }

    pub fn update(&mut self, mouse_pos: glam::Vec2) {
        let surface_size = self.renderer.surface_size();
        let surface_size = glam::Vec2::new(surface_size.width as f32, surface_size.height as f32);

        let rot_range = glam::Vec2::new(std::f32::consts::PI, std::f32::consts::FRAC_PI_2);

        if let Some(camera) = self.world.get_mut::<Camera>(self.camera_entity) {
            let norm_orbit_angle = (mouse_pos / surface_size) * 2.0 - 1.0;
            let orbit_angle = norm_orbit_angle * rot_range;
            let new_camera = Camera {
                orbit_angle,
                ..*camera
            };
            *camera = new_camera;
        }
    }
}
