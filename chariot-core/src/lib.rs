pub mod entity_location;
pub mod lap_info;
pub mod networking;
pub mod physics_changes;
pub mod player_inputs;
pub mod questions;
mod settings;

pub use settings::GLOBAL_CONFIG;

pub type PlayerID = usize;
