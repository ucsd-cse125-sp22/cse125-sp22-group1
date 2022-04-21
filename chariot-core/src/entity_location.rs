use glam::DVec3;

// EntityLocation gets sent back from the server to the client to give it
// information on the results of the simulation == where to render players
pub struct EntityLocation {
    pub position: DVec3,
    pub unit_steer_direction: DVec3, // should be a normalized vector
    pub unit_upward_direction: DVec3, // should be a normalized vector
}
