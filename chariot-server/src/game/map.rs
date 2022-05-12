use serde_json::Value;

use crate::checkpoints::*;

pub struct Map {
    pub major_zones: Vec<MajorCheckpoint>,
    pub checkpoints: Vec<MinorCheckpoint>,
    pub finish_line: FinishLine,
}

impl Map {
    //pub fn load(filename: &str) -> Result<Map, gltf::Error> {
    pub fn load(filename: &str) {
        println!(
            "loading {}, please give a sec I swear it's not lagging",
            filename
        );
        let model_name = filename.split(".").next().expect("invalid filename format");
        let (document, buffers, images) = gltf::import(filename).unwrap();

        //let mut mesh_handles = Vec::new();
        for (mesh_idx, mesh) in document.meshes().enumerate() {
            println!(
                "processing mesh {}",
                mesh.name().unwrap_or("[a mesh that's not named]")
            );

            if mesh.primitives().len() != 1 {
                print!(
                    "Warning: I'm expecting one prim per mesh so things might not work properly"
                );
            }

            for (prim_idx, primitive) in mesh.primitives().enumerate() {
                println!("\tprocessing prim {}", prim_idx);
                //let handle = self.import_mesh(renderer, &buffers, &primitive);
                //mesh_handles.push(handle);
            }

            if let Some(extras) = mesh.extras().as_ref() {
                let mesh_data: Value = serde_json::from_str(extras.as_ref().get()).unwrap();
                println!("X: {:?}", mesh_data["render"] == 0);
            }
        }

        println!("done!");

        // core::result::Result::Ok(Map {
        // 	major_zones = Vec::new(),
        // 	checkpoints = Vec::new(),
        // 	finish_line = FinishLine::new(),
        // })
    }
}
