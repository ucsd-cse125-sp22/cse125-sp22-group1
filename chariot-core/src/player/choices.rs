use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerChoices {
    pub chair: Chair,
    pub map: Track,
    pub ready: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Chair {
    Standard,
}

impl fmt::Display for Chair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            Chair::Standard => "standard",
        };
        write!(f, "{}", printable)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Track {
    Track,
}

impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            Track::Track => "track",
        };
        write!(f, "{}", printable)
    }
}

impl Default for PlayerChoices {
    fn default() -> Self {
        Self {
            chair: Chair::Standard,
            map: Track::Track,
            ready: false,
        }
    }
}
