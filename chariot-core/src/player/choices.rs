use serde::{Deserialize, Serialize};

use crate::GLOBAL_CONFIG;

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerChoices {
    pub chair: String,
    pub map: String,
    pub ready: bool,
}

impl Default for PlayerChoices {
    fn default() -> Self {
        Self {
            chair: GLOBAL_CONFIG.default_chair.clone(),
            map: GLOBAL_CONFIG.default_map_vote.clone(),
            ready: false,
        }
    }
}
