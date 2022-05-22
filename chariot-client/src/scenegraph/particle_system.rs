use std::ops::Bound;

use super::{Transform, World};
use crate::drawable::particle::ParticleDrawable;
use crate::drawable::Drawable;
use crate::renderer::render_job::RenderGraph;
use crate::renderer::Renderer;
use crate::resources::{
    material::MaterialBuilder, MaterialHandle, ResourceManager, StaticMeshHandle, TextureHandle,
};
use crate::util::{Pcg32Rng, Rng};

pub struct BillboardParticle {
    vel: glam::Vec3,
    pos: glam::Vec3,
    size: glam::Vec2,
    lifetime: f32,
}

impl Default for BillboardParticle {
    fn default() -> Self {
        Self {
            vel: glam::Vec3::ZERO,
            pos: glam::Vec3::ZERO,
            size: glam::Vec2::ONE,
            lifetime: 0.0,
        }
    }
}

pub struct RotatedParticle {
    vel: glam::Vec3,
    pos: glam::Vec3,
    size: glam::Vec2,
    rot: glam::Quat,
    lifetime: f32,
}

impl Default for RotatedParticle {
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
    pub has_gravity: bool,
}

pub struct ParticleSystem {
    material_handle: MaterialHandle,
    mesh_handle: StaticMeshHandle,
    pos_range: (glam::Vec3, glam::Vec3),
    size_range: (glam::Vec2, glam::Vec2),
    initial_vel: glam::Vec3,
    spawn_rate: f32,
    lifetime: f32,
    rotation: ParticleRotation,
    has_gravity: bool,
    rng: Pcg32Rng,
}

impl ParticleSystem {
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
            has_gravity: params.has_gravity,
            rng: Pcg32Rng::default(),
        }
    }

    pub fn spawn(
        &mut self,
        renderer: &Renderer,
        world: &mut World,
        transform: &Transform,
        delta_time: f32,
    ) {
        let (pos_low, pos_high) = self.pos_range;
        let (size_low, size_high) = self.size_range;

        let u: f32 = self.rng.next();
        let world_root = world.root();
        let spawn_count_f32 = self.spawn_rate * delta_time;
        let spawn_count = (self.spawn_rate * delta_time) as usize
            + if u < f32::fract(spawn_count_f32) {
                1
            } else {
                0
            };

        println!("Spawning {} particles", spawn_count);
        for _ in 0..spawn_count {
            let vel = self.initial_vel;
            let pos = transform.translation + sample_bound(&mut self.rng, pos_low, pos_high);
            let size = sample_bound(&mut self.rng, size_low, size_high);

            let drawable = ParticleDrawable::new(renderer, self.mesh_handle, self.material_handle);

            match self.rotation {
                ParticleRotation::Billboard => world
                    .builder()
                    .attach(world_root)
                    .with(BillboardParticle {
                        vel,
                        pos,
                        size,
                        lifetime: self.lifetime,
                    })
                    .with(Some(drawable))
                    .build(),
                ParticleRotation::Random => {
                    let rot: glam::Quat = self.rng.next();
                    world
                        .builder()
                        .attach(world_root)
                        .with(RotatedParticle {
                            vel,
                            pos,
                            size,
                            rot,
                            lifetime: self.lifetime,
                        })
                        .with(Some(drawable))
                        .build()
                }
            };
        }
    }

    pub fn update(world: &mut World, delta_time: f32) {
        let mut to_remove = vec![];
        if let Some(particles) = world.storage_mut::<BillboardParticle>() {
            for (entity, particle) in particles.iter_with_entity_mut() {
                particle.pos += delta_time * particle.vel;
                particle.lifetime -= delta_time;
                if particle.lifetime < 0.0 {
                    to_remove.push(*entity);
                }
            }
        }

        for entity in to_remove {
            world.remove::<BillboardParticle>(entity);
            world.remove::<Option<ParticleDrawable>>(entity);
        }

        to_remove = vec![];
        if let Some(particles) = world.storage_mut::<RotatedParticle>() {
            for (entity, particle) in particles.iter_with_entity_mut() {
                particle.pos += delta_time * particle.vel;
                particle.lifetime -= delta_time;
                if particle.lifetime < 0.0 {
                    to_remove.push(*entity);
                }
            }
        }

        for entity in to_remove {
            world.remove::<RotatedParticle>(entity);
            world.remove::<Option<ParticleDrawable>>(entity);
        }
    }

    pub fn render_graphs<'a>(
        world: &'a World,
        renderer: &Renderer,
        resources: &'a ResourceManager,
        view: glam::Mat4,
        proj: glam::Mat4,
    ) -> Vec<RenderGraph<'a>> {
        let view_right = view.row(0);
        let view_up = view.row(1);
        let rot_to_view = glam::Mat4::from_cols(view_right, view_up, glam::Vec4::Z, glam::Vec4::W);

        let mut render_graphs = vec![];
        if let Some(drawables) = world.storage::<Option<ParticleDrawable>>() {
            for (entity, drawable) in drawables
                .iter_with_entity()
                .filter_map(|(e, d)| d.as_ref().map(|d| (e, d)))
            {
                let model = if let Some(particle) = world.get::<BillboardParticle>(*entity) {
                    glam::Mat4::from_scale_rotation_translation(
                        glam::Vec3::from((particle.size, 1.0)),
                        glam::Quat::IDENTITY,
                        particle.pos,
                    ) * rot_to_view
                } else if let Some(particle) = world.get::<RotatedParticle>(*entity) {
                    glam::Mat4::from_scale_rotation_translation(
                        glam::Vec3::from((particle.size, 1.0)),
                        particle.rot,
                        particle.pos,
                    )
                } else {
                    glam::Mat4::IDENTITY
                };

                drawable.update_mvp(renderer, model, view, proj);
                let graph = drawable.render_graph(resources);
                render_graphs.push(graph);
            }
        }

        println!("Rendering {} particles", render_graphs.len());
        render_graphs
    }
}
