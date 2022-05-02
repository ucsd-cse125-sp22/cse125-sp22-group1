use chariot_core::entity_location::EntityLocation;

use crate::drawable::*;
use crate::renderer::*;
use crate::resources::*;
use crate::scenegraph::*;

pub struct GraphicsManager {
    pub world: World,
    pub renderer: Renderer,
    pub resources: ResourceManager,

    player_ids: [Option<u64>; 4],
    next_entity_id: u64,
}

impl GraphicsManager {
    pub fn new(mut renderer: Renderer) -> Self {
        renderer.register_pass(
            "boring",
            &direct_graphics_depth_pass!(include_str!("shader.wgsl")),
        );

        renderer.register_pass(
            "forward",
            &indirect_graphics_depth_pass!(
                include_str!("shader.wgsl"),
                [wgpu::TextureFormat::Rgba16Float]
            ),
        );

        renderer.register_pass(
            "postprocess",
            &direct_graphics_nodepth_pass!(include_str!("postprocess.wgsl")),
        );

        let (depth_tex, color_tex, fb_desc) =
            depth_color_framebuffer(&renderer, wgpu::TextureFormat::Rgba16Float);
        renderer.register_framebuffer("forward_out", fb_desc, [depth_tex, color_tex]);

        let mut resources = ResourceManager::new();
        let mut world = World::new();

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
            player_ids: [Some(0), None, None, None],
            next_entity_id: 1,
        }
    }

    pub fn add_player(&mut self, player_num: u8, is_self: bool) {
        let chair_import_result = self
            .resources
            .import_gltf(&mut self.renderer, "models/chair.glb");

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

        // Only follow the new chair around if this is us
        if is_self {
            chair.set_component(Camera {
                orbit_angle: glam::Vec2::ZERO,
                distance: 2.0,
            });
        }

        // But all chairs get displayed and easily indexed
        chair.set_component(EntityID {
            id: self.next_entity_id,
        });

        self.player_ids[player_num as usize] = Some(self.next_entity_id);
        self.next_entity_id += 1;

        self.world.root_mut().add_child(chair);

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
            translation: location.position.as_vec3(),
            rotation: rotation_1.mul_quat(rotation_2),
            // only works for chairs! do something more robust for other entities later
            scale: glam::vec3(1.1995562314987183, 2.2936718463897705, 1.1995562314987183) * 0.2,
        };
    }

    pub fn update_player_location(&mut self, location: &EntityLocation, player_num: u8) {
        if self.player_ids[player_num as usize].is_none() {
            self.add_player(player_num, false);
        }
        let id = self.player_ids[player_num as usize].unwrap();
        dfs_mut(self.world.root_mut(), &|e| {
            if let Some(entity_id) = e.get_component::<EntityID>() {
                if entity_id.id == id {
                    println!("new location for #{}: {}", player_num, location.position);
                    println!(
                        "new steer direction for #{}: {}",
                        player_num, location.unit_steer_direction
                    );
                    println!(
                        "new upward direction for #{}: {}",
                        player_num, location.unit_upward_direction
                    );
                    e.set_component(GraphicsManager::EntityLocation_to_Transform(&location))
                }
            }
        });
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
            cur_model_transform.rotation = glam::Quat::IDENTITY;

            let cur_model = cur_model_transform.to_mat4();

            let acc_model = *acc * cur_model;

            if let Some(camera) = e.get_component::<Camera>() {
                view_local = camera.view_mat4();
                view_global = acc_model;
            }

            acc_model
        });

        let view = view_local * view_global.inverse();

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

    pub fn update(&mut self, mouse_pos: glam::Vec2) {
        let surface_size = self.renderer.surface_size();
        let surface_size = glam::Vec2::new(surface_size.width as f32, surface_size.height as f32);

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
    }
}
