use std::collections::VecDeque;

use glam::DVec2;
use serde_json::Value;

use crate::{
    checkpoints::*,
    physics::{bounding_box::BoundingBox, ramp::Ramp, trigger_entity::TriggerEntity},
};
use chariot_core::{player::lap_info::ZoneID, GLOBAL_CONFIG};

use super::powerup::pickups::ItemBox;

pub struct Map {
    // Something you cannot pass through/has collision
    pub colliders: Vec<BoundingBox>,
    pub ramps: Vec<Ramp>,

    // basically: while you're on the track, you should get a speedup (vroom vroom zoom zoom)
    pub speedup_zones: Vec<BoundingBox>,

    // Map's checkpoints, which track progress through the track
    pub checkpoints: Vec<Checkpoint>,

    // Map's zones, which divide the map into large blocks you must pass through
    pub major_zones: Vec<Zone>,

    // Map's finish line, which... is the finish line
    pub finish_line: FinishLine,

    pub powerups: Vec<ItemBox>,
}

fn import_mesh(
    buffers: &[gltf::buffer::Data],
    primitive: &gltf::Primitive,
    transform: glam::Mat4,
) -> BoundingBox {
    let mut bounds = BoundingBox::extremes();

    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
    let vert_iter = reader
        .read_positions()
        .expect("Couldn't read primitive's positions!");
    let mut vert_buf = vert_iter.collect::<Vec<[f32; 3]>>();

    for vertex in vert_buf.iter_mut() {
        *vertex = transform
            .transform_point3(glam::Vec3::from_slice(vertex))
            .to_array();
    }

    let glam_verts = vert_buf.iter().map(|e| glam::Vec3::from_slice(e));

    bounds = bounds.accum(BoundingBox::from_vecs(
        glam_verts
            .clone()
            .reduce(|a, e| a.min(e))
            .unwrap()
            .as_dvec3(),
        glam_verts
            .clone()
            .reduce(|a, e| a.max(e))
            .unwrap()
            .as_dvec3(),
    ));

    bounds
}

impl Map {
    pub fn load(filename: String) -> core::result::Result<Map, gltf::Error> {
        println!(
            "loading {}, please give a sec I swear it's not lagging",
            filename
        );
        filename.split(".").next().expect("invalid filename format");
        let map_path = format!("{}/{}.glb", GLOBAL_CONFIG.tracks_folder, filename);
        let (document, buffers, _) = gltf::import(map_path)?;
        if document.scenes().count() != 1 {
            panic!(
                "Document {} has {} scenes!",
                filename,
                document.scenes().count()
            );
        }

        let mut colliders: Vec<BoundingBox> = Vec::new();
        let mut ramps: Vec<Ramp> = Vec::new();

        let mut speedup_zones: Vec<BoundingBox> = Vec::new();

        let mut checkpoints: Vec<Checkpoint> = Vec::new();
        let mut major_zones: Vec<Zone> = Vec::new();
        let mut last_zone: ZoneID = 0;

        let mut finish_line: Option<FinishLine> = None;
        let mut world_bounds = BoundingBox::extremes();

        let mut powerups = Vec::new();

        // Queue of (Node, Transformation) tuples
        let mut queue: VecDeque<(gltf::Node, glam::Mat4)> = document
            .scenes()
            .next()
            .expect("No root node in scene")
            .nodes()
            .map(|n| (n, glam::Mat4::IDENTITY))
            .collect::<VecDeque<(gltf::Node, glam::Mat4)>>();

        // Probably better to do this recursively but i didn't wanna change stuff like crazy, not that it really matters since this is just loading anyways
        while let Some((node, parent_transform)) = queue.pop_front() {
            let transform = parent_transform
                * (match node.transform() {
                    gltf::scene::Transform::Matrix { matrix } => {
                        glam::Mat4::from_cols_array_2d(&matrix)
                    }
                    gltf::scene::Transform::Decomposed {
                        translation,
                        rotation,
                        scale,
                    } => glam::Mat4::from_scale_rotation_translation(
                        glam::Vec3::from(scale),
                        glam::Quat::from_array(rotation),
                        glam::Vec3::from(translation),
                    ),
                });

            if let Some(mesh) = node.mesh() {
                if let Some(extras) = mesh.extras().as_ref() {
                    let mesh_data: Value = serde_json::from_str(extras.as_ref().get()).unwrap();
                    if let Some(Value::String(purpose)) = mesh_data.get("purpose") {
                        for (_, primitive) in mesh.primitives().enumerate() {
                            let mesh_bounds = import_mesh(&buffers, &primitive, transform);

                            if purpose == "trigger" {
                                if let Some(Value::String(trigger_type)) = mesh_data.get("trigger")
                                {
                                    if trigger_type == "checkpoint" {
                                        let idx = mesh_data
                                            .get("checkpoint_id")
                                            .unwrap()
                                            .as_u64()
                                            .unwrap();
                                        println!(
                                            "Loading mesh '{}' as a trigger_checkpoint_{}",
                                            mesh.name().unwrap_or("<unnamed>"),
                                            idx
                                        );
                                        checkpoints.push(Checkpoint::new(idx, mesh_bounds));
                                    } else if trigger_type == "zone" {
                                        let idx =
                                            mesh_data.get("zone_id").unwrap().as_u64().unwrap();
                                        println!(
                                            "Loading mesh '{}' as a trigger_zone_{}",
                                            mesh.name().unwrap_or("<unnamed>"),
                                            idx
                                        );
                                        last_zone = idx.max(last_zone);
                                        major_zones.push(Zone::new(idx, mesh_bounds));
                                    } else if trigger_type == "finish_line" {
                                        println!(
                                            "Loading mesh '{}' as a trigger_finish_line",
                                            mesh.name().unwrap_or("<unnamed>")
                                        );
                                        finish_line = Some(FinishLine::new(mesh_bounds, 1));
                                        // } else if trigger_type == "powerup" {
                                    } else if trigger_type == "powerup" {
                                        println!(
                                            "Loading mesh '{}' as a trigger_powerup",
                                            mesh.name().unwrap_or("<unnamed>")
                                        );
                                        powerups.push(ItemBox::new(mesh_bounds));
                                    } else {
                                        panic!("Unknown trigger type '{}'!", trigger_type);
                                    }
                                }
                            } else if purpose == "collision" {
                                println!(
                                    "Loading mesh '{}' as a collider",
                                    mesh.name().unwrap_or("<unnamed>")
                                );
                                colliders.push(mesh_bounds);
                            } else if purpose == "ramp" {
                                let incline_direction: String = mesh_data
                                    .get("incline_direction")
                                    .expect("Ramps should have an incline direction!")
                                    .to_string();

                                let incline_direction_vec = if incline_direction == "pos_x" {
                                    -1.0 * DVec2::X
                                } else if incline_direction == "neg_x" {
                                    DVec2::X
                                } else if incline_direction == "pos_z" {
                                    -1.0 * DVec2::Y
                                } else {
                                    DVec2::Y
                                };

                                let ramp = Ramp {
                                    footprint: [
                                        [mesh_bounds.min_x, mesh_bounds.max_x],
                                        [mesh_bounds.min_z, mesh_bounds.max_z],
                                    ],
                                    min_height: mesh_bounds.min_y,
                                    max_height: mesh_bounds.max_y,
                                    incline_direction: incline_direction_vec,
                                };
                                ramps.push(ramp);
                            } else if purpose == "speedup" {
                                speedup_zones.push(mesh_bounds);
                            } else {
                                // panic!(
                                //     "Mesh '{}' has unknown purpose '{}'!",
                                //     mesh.name().unwrap_or("<unnamed>"),
                                //     purpose
                                // );
                            }

                            world_bounds = world_bounds.accum(mesh_bounds);
                        }
                    }
                }
            }

            for child in node.children() {
                queue.push_back((child, transform));
            }
        }

        println!("done!");

        core::result::Result::Ok(Self {
            colliders,
            ramps,
            speedup_zones,
            checkpoints,
            major_zones,
            finish_line: finish_line
                .expect(format!("Map {} has no finish line!", filename).as_str())
                .set_last_zone(last_zone),
            powerups,
        })
    }

    // good god figuring out type stuff here made me want to pivot to javascript permanently
    pub fn trigger_iter(&mut self) -> impl Iterator<Item = &mut dyn TriggerEntity> {
        self.checkpoints
            .iter_mut()
            .map(|c| c as &mut dyn TriggerEntity)
            .chain(
                self.major_zones
                    .iter_mut()
                    .map(|z| z as &mut dyn TriggerEntity),
            )
            .chain(std::iter::once(
                &mut self.finish_line as &mut dyn TriggerEntity,
            ))
    }
}
