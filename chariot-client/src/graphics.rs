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
        let mut world = World::new();

        let import_result = resources.import_gltf(&mut renderer, "models/DamagedHelmet.glb");

        let mut helmet = Entity::new();
        helmet.set_component(Transform {
            translation: glam::Vec3::ZERO,
            rotation: glam::Quat::from_axis_angle(glam::Vec3::X, f32::to_radians(90.0)),
            scale: glam::vec3(0.3, 0.3, 0.3),
        });

        helmet.set_component(import_result.expect("Failed to import model").drawables);

        helmet.set_component(EntityID { id: 0 });

        world.root_mut().add_child(helmet);

        Self {
            world: world,
            renderer: renderer,
            resources: resources,
            player_ids: [Some(0), None, None, None],
            next_entity_id: 1,
        }
    }

    pub fn add_player(&mut self, player_num: u8) {
        // still using this as the placeholder asset
        let import_result = self
            .resources
            .import_gltf(&mut self.renderer, "models/DamagedHelmet.glb");

        let mut helmet = Entity::new();
        helmet.set_component(Transform {
            translation: glam::Vec3::ZERO,
            rotation: glam::Quat::from_axis_angle(glam::Vec3::X, f32::to_radians(90.0)),
            scale: glam::vec3(0.3, 0.3, 0.3),
        });

        helmet.set_component(import_result.expect("Failed to import model").drawables);

        helmet.set_component(EntityID {
            id: self.next_entity_id,
        });

        self.player_ids[player_num as usize] = Some(self.next_entity_id);
        self.next_entity_id += 1;

        self.world.root_mut().add_child(helmet);
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
            scale: glam::Vec3::ONE,
        };
    }

    pub fn update_player_location(&mut self, location: &EntityLocation, player_num: u8) {
        if let Some(id) = self.player_ids[player_num as usize] {
            dfs_mut(self.world.root_mut(), &|e| {
                if let Some(entity_id) = e.get_component::<EntityID>() {
                    if entity_id.id == id {
                        e.set_component(GraphicsManager::EntityLocation_to_Transform(&location))
                    }
                }
            });
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
