use std::ops::Bound;

use super::{Entity, Transform, World};
use crate::drawable::particle::ParticleDrawable;
use crate::drawable::Drawable;
use crate::renderer::render_job::RenderGraph;
use crate::renderer::Renderer;
use crate::resources::Handle;
use crate::resources::{
    material::MaterialBuilder, MaterialHandle, ResourceManager, StaticMeshHandle, TextureHandle,
};
use crate::scenegraph::NULL_ENTITY;
use crate::util::{Pcg32Rng, Rng};

pub struct BillboardParticle<const ID: u32> {
    vel: glam::Vec3,
    pos: glam::Vec3,
    size: glam::Vec2,
    lifetime: f32,
}

impl<const ID: u32> Default for BillboardParticle<ID> {
    fn default() -> Self {
        Self {
            vel: glam::Vec3::ZERO,
            pos: glam::Vec3::ZERO,
            size: glam::Vec2::ONE,
            lifetime: 0.0,
        }
    }
}

pub struct RotatedParticle<const ID: u32> {
    vel: glam::Vec3,
    pos: glam::Vec3,
    size: glam::Vec2,
    rot: glam::Quat,
    lifetime: f32,
}

impl<const ID: u32> Default for RotatedParticle<ID> {
    fn default() -> Self {
        Self {
            vel: glam::Vec3::ZERO,
            pos: glam::Vec3::ZERO,
            size: glam::Vec2::ONE,
            rot: glam::Quat::IDENTITY,
            lifetime: 0.0,
        }
    }
}

fn sample_bound<R, T>(rng: &mut R, low: T, high: T) -> T
where
    R: Rng<T>,
    T: std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::ops::Mul<Output = T> + Copy,
{
    (high - low.clone()) * rng.next() + low.clone()
}

pub enum ParticleRotation {
    Billboard,
    Random,
    RandomAroundAxis(glam::Vec3),
    Constant(glam::Quat),
}

pub struct ParticleSystemParams {
    pub texture_handle: TextureHandle,
    pub mesh_handle: StaticMeshHandle,
    pub pos_range: (glam::Vec3, glam::Vec3),
    pub size_range: (glam::Vec2, glam::Vec2),
    pub initial_vel: glam::Vec3,
    pub spawn_rate: f32,
    pub lifetime: f32,
    pub rotation: ParticleRotation,
    pub gravity: f32,
}

impl Default for ParticleSystemParams {
    fn default() -> Self {
        Self {
            texture_handle: TextureHandle::INVALID,
            mesh_handle: StaticMeshHandle::INVALID,
            pos_range: (glam::Vec3::ZERO, glam::Vec3::ZERO),
            size_range: (glam::Vec2::ONE, glam::Vec2::ONE),
            initial_vel: glam::Vec3::ZERO,
            spawn_rate: 0.0,
            lifetime: 0.0,
            rotation: ParticleRotation::Billboard,
            gravity: 0.0,
        }
    }
}

pub struct ParticleSystem<const ID: u32> {
    material_handle: MaterialHandle,
    mesh_handle: StaticMeshHandle,
    pos_range: (glam::Vec3, glam::Vec3),
    size_range: (glam::Vec2, glam::Vec2),
    initial_vel: glam::Vec3,
    spawn_rate: f32,
    lifetime: f32,
    rotation: ParticleRotation,
    gravity: f32,
    rng: Pcg32Rng,
}

impl<const ID: u32> ParticleSystem<ID> {
    pub fn register_components(world: &mut World) {
        world.register::<Option<ParticleDrawable>>();
        world.register::<BillboardParticle<ID>>();
        world.register::<RotatedParticle<ID>>();
    }

    pub fn new(
        renderer: &Renderer,
        resources: &mut ResourceManager,
        params: ParticleSystemParams,
    ) -> Self {
        let texture_view = resources
            .textures
            .get(&params.texture_handle)
            .unwrap()
            .create_view(&wgpu::TextureViewDescriptor::default());

        let material = MaterialBuilder::new(renderer, "particle")
            .texture_resource(1, 0, texture_view)
            .produce();
        let material_handle = resources.register_material(material);
        Self {
            material_handle,
            mesh_handle: params.mesh_handle,
            pos_range: params.pos_range,
            size_range: params.size_range,
            initial_vel: params.initial_vel,
            spawn_rate: params.spawn_rate,
            lifetime: params.lifetime,
            rotation: params.rotation,
            gravity: params.gravity,
            rng: Pcg32Rng::default(),
        }
    }

    pub fn spawn(
        &mut self,
        renderer: &Renderer,
        world: &mut World,
        transform: &Transform,
        attach: Entity,
        delta_time: f32,
    ) {
        let (pos_low, pos_high) = self.pos_range;
        let (size_low, size_high) = self.size_range;

        let u: f32 = self.rng.next();
        let spawn_count_f32 = self.spawn_rate * delta_time;
        let spawn_count = (self.spawn_rate * delta_time) as usize
            + if u < f32::fract(spawn_count_f32) {
                1
            } else {
                0
            };

        for _ in 0..spawn_count {
            let vel = self.initial_vel;
            let pos = transform.translation + sample_bound(&mut self.rng, pos_low, pos_high);
            let size =
                sample_bound(&mut self.rng, size_low, size_high) * transform.scale.truncate();

            let drawable = ParticleDrawable::new(renderer, self.mesh_handle, self.material_handle);

            match self.rotation {
                ParticleRotation::Billboard => world
                    .builder()
                    .attach(attach)
                    .with(BillboardParticle::<ID> {
                        vel,
                        pos,
                        size,
                        lifetime: self.lifetime,
                    })
                    .with(Some(drawable))
                    .build(),
                ParticleRotation::Random => {
                    let rand_rot: glam::Quat = self.rng.next();
                    let rot: glam::Quat = transform.rotation * rand_rot;
                    world
                        .builder()
                        .attach(attach)
                        .with(RotatedParticle::<ID> {
                            vel,
                            pos,
                            size,
                            rot,
                            lifetime: self.lifetime,
                        })
                        .with(Some(drawable))
                        .build()
                }
                ParticleRotation::RandomAroundAxis(axis) => {
                    let v: f32 = self.rng.next();
                    let rand_angle = v * std::f32::consts::PI;
                    let rot = glam::Quat::from_axis_angle(axis, rand_angle);
                    world
                        .builder()
                        .attach(attach)
                        .with(RotatedParticle::<ID> {
                            vel,
                            pos,
                            size,
                            rot,
                            lifetime: self.lifetime,
                        })
                        .with(Some(drawable))
                        .build()
                }
                ParticleRotation::Constant(rot) => world
                    .builder()
                    .attach(attach)
                    .with(RotatedParticle::<ID> {
                        vel,
                        pos,
                        size,
                        rot,
                        lifetime: self.lifetime,
                    })
                    .with(Some(drawable))
                    .build(),
            };
        }
    }

    pub fn update(&self, world: &mut World, delta_time: f32) {
        let mut to_remove = vec![];
        if let Some(particles) = world.storage_mut::<BillboardParticle<ID>>() {
            for (entity, particle) in particles.iter_with_entity_mut() {
                particle.vel += self.gravity * -glam::Vec3::Y * delta_time;
                particle.pos += particle.vel * delta_time;
                particle.lifetime -= delta_time;
                if particle.lifetime < 0.0 {
                    to_remove.push(*entity);
                }
            }
        }

        for entity in to_remove {
            world.remove::<BillboardParticle<ID>>(entity);
            world.remove::<Option<ParticleDrawable>>(entity);
        }

        to_remove = vec![];
        if let Some(particles) = world.storage_mut::<RotatedParticle<ID>>() {
            for (entity, particle) in particles.iter_with_entity_mut() {
                particle.vel += self.gravity * -glam::Vec3::Y * delta_time;
                particle.pos += delta_time * particle.vel;
                particle.lifetime -= delta_time;
                if particle.lifetime < 0.0 {
                    to_remove.push(*entity);
                }
            }
        }

        for entity in to_remove {
            world.remove::<RotatedParticle<ID>>(entity);
            world.remove::<Option<ParticleDrawable>>(entity);
        }
    }

    pub fn calc_particle_model(
        &self,
        world: &World,
        entity: Entity,
        view: glam::Mat4,
    ) -> Option<glam::Mat4> {
        let (view_scale, view_rot, view_trans) = view.inverse().to_scale_rotation_translation();

        if let Some(particle) = world.get::<BillboardParticle<ID>>(entity) {
            Some(glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::from((particle.size, 1.0)),
                view_rot,
                particle.pos,
            ))
        } else if let Some(particle) = world.get::<RotatedParticle<ID>>(entity) {
            Some(glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::from((particle.size, 1.0)),
                particle.rot,
                particle.pos,
            ))
        } else {
            None
        }
    }
}
