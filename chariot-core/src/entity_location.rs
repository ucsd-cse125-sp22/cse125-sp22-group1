use glam::DVec3;
use serde::{Deserialize, Serialize};

pub type Bounds = (glam::Vec3, glam::Vec3);

pub fn new_bounds() -> Bounds {
    let low_bound = glam::vec3(f32::MAX, f32::MAX, f32::MAX);
    let high_bound = glam::vec3(f32::MIN, f32::MIN, f32::MIN);
    (low_bound, high_bound)
}

pub fn accum_bounds(mut acc: Bounds, new: Bounds) -> Bounds {
    acc.0 = acc.0.min(new.0);
    acc.1 = acc.1.max(new.1);
    acc
}

// EntityLocation gets sent back from the server to the client to give it
// information on the results of the simulation == where to render players
#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct EntityLocation {
    pub position: DVec3,
    pub unit_steer_direction: DVec3,  // should be a normalized vector
    pub unit_upward_direction: DVec3, // should be a normalized vector
}
