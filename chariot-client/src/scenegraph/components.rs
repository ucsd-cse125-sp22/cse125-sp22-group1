use chariot_core::entity_location::EntityLocation;
use crate::scenegraph::*;

// ---------- Components ---------- //

#[derive(Clone, Copy)]
pub struct SceneNode {
    pub first: Entity,
    pub next: Entity,
    pub prev: Entity,
    pub parent: Entity,
}

impl Default for SceneNode {
    fn default() -> Self {
        Self {
            first: NULL_ENTITY,
            next: NULL_ENTITY,
            prev: NULL_ENTITY,
            parent: NULL_ENTITY,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Transform {
    pub translation: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Transform {
    pub fn from_entity_location(entity_location: &EntityLocation) -> Transform {
        let rotation_1 = glam::Quat::from_rotation_arc(
            glam::Vec3::Z,
            entity_location.unit_steer_direction.normalize().as_vec3(),
        );
        let rotation_2 = glam::Quat::from_rotation_arc(
            glam::Vec3::Y,
            entity_location.unit_upward_direction.normalize().as_vec3(),
        );

        Transform {
            translation: entity_location.position.as_vec3() + glam::Vec3::new(0.0, 1.0, 0.0),
            rotation: rotation_1.mul_quat(rotation_2),
            // only works for chairs! do something more robust for other entities later
            scale: glam::vec3(1.1995562314987183, 2.2936718463897705, 1.1995562314987183) * 0.2,
        }
    }

    pub fn to_mat4(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Camera {
    pub orbit_angle: glam::Vec2,
    pub distance: f32,
}

impl Camera {
    pub fn view_mat4(&self) -> glam::Mat4 {
        let look_rot = glam::Quat::from_euler(
            glam::EulerRot::YXZ,
            self.orbit_angle.x,
            std::f32::consts::PI - self.orbit_angle.y,
            0.0,
        );
        let look_dir = look_rot * glam::Vec3::Z;
        let look_offset = look_dir * self.distance;

        glam::Mat4::look_at_rh(look_offset, glam::Vec3::ZERO, glam::Vec3::Y)
    }
}

#[derive(Default, Clone)]
pub struct Light {
    pub dir: glam::Vec3,
    pub framebuffer_name: String,
}

impl Light {
    pub fn new_directional(dir: glam::Vec3, bounds: Bounds) -> Self {
        Self {
            dir,
            framebuffer_name: "shadow_out1".to_string(),
        }
    }

    pub fn calc_view_proj(&self, bounds: &Bounds) -> (glam::Mat4, glam::Mat4) {
        let scene_center = (bounds.0 + bounds.1) * 0.5;
        let scene_radius = (bounds.1 - scene_center).length();

        let dist_padding = 0.0;

        let light_pos = scene_center - self.dir * (scene_radius + dist_padding);
        let view = glam::Mat4::look_at_rh(light_pos, scene_center, glam::Vec3::Y);
        let proj = glam::Mat4::orthographic_rh(
            -scene_radius,
            scene_radius,
            -scene_radius,
            scene_radius,
            0.01,
            scene_radius * 2.0,
        );

        (view, proj)
    }
}
