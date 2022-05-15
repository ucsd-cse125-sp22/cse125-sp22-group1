use std::collections::VecDeque;

use serde_json::Value;

use crate::checkpoints::*;
use chariot_core::entity_location::{accum_bounds, new_bounds, Bounds};
use chariot_core::GLOBAL_CONFIG;

pub struct Map {
    pub colliders: Vec<Bounds>,
    pub checkpoints: Vec<Checkpoint>,
    pub major_zones: Vec<Zone>,
    pub finish_line: FinishLine,
}

fn import_mesh(
    buffers: &[gltf::buffer::Data],
    primitive: &gltf::Primitive,
    transform: glam::Mat4,
) -> Bounds {
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

        let colliders = Vec::new();
        let checkpoints = Vec::new();
        let major_zones: Vec<Zone> = Vec::new();
        let finish_line: Option<FinishLine> = None;
        let mut world_bounds = new_bounds();

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
                    if mesh_data["collide"] == 1 {
                        println!("processing mesh '{}'", mesh.name().unwrap_or("<unnamed>"));
                        for (prim_idx, primitive) in mesh.primitives().enumerate() {
                            println!("\tprocessing prim {}", prim_idx);

                            let mesh_bounds = import_mesh(&buffers, &primitive, transform);

                            //mesh_handles.push(mesh_handle);
                            bounds = accum_bounds(bounds, mesh_bounds);
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
            colliders: todo!(),
            checkpoints: todo!(),
            major_zones: todo!(),
            finish_line: todo!(),
        })
    }
}
