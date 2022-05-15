use std::collections::VecDeque;

use glam::dvec3;
use serde_json::Value;

use crate::{
    checkpoints::*,
    physics::{bounding_box::BoundingBox, trigger_entity::TriggerEntity},
};
use chariot_core::GLOBAL_CONFIG;

pub struct Map {
    // Something you cannot pass through/has collision
    pub colliders: Vec<BoundingBox>,

    // Map's checkpoints, which track progress through the track
    pub checkpoints: Vec<Checkpoint>,

    // Map's zones, which divide the map into large blocks you must pass through
    pub major_zones: Vec<Zone>,

    // Map's finish line, which... is the finish line
    pub finish_line: FinishLine,
}

fn import_mesh(
    buffers: &[gltf::buffer::Data],
    primitive: &gltf::Primitive,
    transform: glam::Mat4,
) -> BoundingBox {
    todo!()
}

impl Map {
    pub fn load(filename: String) -> core::result::Result<Map, gltf::Error> {
        println!(
            "loading {}, please give a sec I swear it's not lagging",
            filename
        );
        let model_name = filename.split(".").next().expect("invalid filename format");
        let resource_path = format!("{}/{}", GLOBAL_CONFIG.resource_folder, filename);
        let (document, buffers, images) = gltf::import(resource_path)?;
        if document.scenes().count() != 1 {
            panic!(
                "Document {} has {} scenes!",
                filename,
                document.scenes().count()
            );
        }

        let colliders: Vec<BoundingBox> = Vec::new();
        let checkpoints: Vec<Checkpoint> = Vec::new();
        let major_zones: Vec<Zone> = Vec::new();
        let mut finish_line: Option<FinishLine> = None;
        let mut world_bounds = BoundingBox::extremes();

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
            //println!("Processing node '{}'", node.name().unwrap_or("<unnamed>"));

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
                        for (prim_idx, primitive) in mesh.primitives().enumerate() {
                            let mesh_bounds = import_mesh(&buffers, &primitive, transform);

                            if purpose == "trigger" {
                                if let Some(Value::String(trigger_type)) =
                                    mesh_data.get("trigger_type")
                                {
                                    println!(
                                        "Loading mesh '{}' as a trigger_{}",
                                        mesh.name().unwrap_or("<unnamed>"),
                                        trigger_type
                                    );

                                    if trigger_type == "checkpoint" {
                                        todo!();
                                    } else if trigger_type == "zone" {
                                        todo!();
                                    } else if trigger_type == "finish_line" {
                                        finish_line = Some(FinishLine::new(
                                            dvec3(0.0, 0.0, 0.0),
                                            dvec3(1.0, 1.0, 1.0),
                                            1,
                                        ));
                                        // } else if trigger_type == "powerup" {
                                    } else {
                                        panic!("Unknown trigger type '{}'!", trigger_type);
                                    }
                                }
                            } else if purpose == "collision" {
                                println!(
                                    "Loading mesh '{}' as a collider",
                                    mesh.name().unwrap_or("<unnamed>")
                                );
                                todo!();
                            } else {
                                panic!(
                                    "Mesh '{}' has unknown purpose '{}'!",
                                    mesh.name().unwrap_or("<unnamed>"),
                                    purpose
                                );
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
            checkpoints,
            major_zones,
            finish_line: finish_line
                .expect(format!("Map {} has no finish line!", filename).as_str()),
        })
    }

    // good god figuring out type stuff here made me want to pivot to javascript permanently
    pub fn trigger_iter(&self) -> impl Iterator<Item = &dyn TriggerEntity> {
        self.checkpoints
            .iter()
            .map(|c| c as &dyn TriggerEntity)
            .chain(self.major_zones.iter().map(|z| z as &dyn TriggerEntity))
            .chain(std::iter::once(&self.finish_line as &dyn TriggerEntity))
    }
}
