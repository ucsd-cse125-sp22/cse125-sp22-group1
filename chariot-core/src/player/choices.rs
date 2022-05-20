use glam::DVec3;
use lazy_static::lazy_static;
use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

use crate::GLOBAL_CONFIG;

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerChoices {
    pub chair: Chair,
    pub map: Track,
    pub ready: bool,
}

impl Default for PlayerChoices {
    fn default() -> Self {
        Self {
            chair: Chair::Recliner,
            map: Track::Track,
            ready: false,
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Chair {
    Swivel,
    Recliner,
    Ergonomic,
    Beanbag,
    Folding,
}

impl Chair {
    pub fn file(&self) -> String {
        match self {
            Chair::Swivel => "swivel",
            Chair::Recliner => "recliner",
            Chair::Ergonomic => "eregonomic",
            Chair::Beanbag => "beanbag",
            Chair::Folding => "folding",
        }
        .to_string()
    }

    fn default_stats() -> HashMap<String, f64> {
        [
            ("gravity_coefficient", GLOBAL_CONFIG.gravity_coefficient),
            ("drag_coefficient", GLOBAL_CONFIG.drag_coefficient),
            (
                "rolling_resistance_coefficient",
                GLOBAL_CONFIG.rolling_resistance_coefficient,
            ),
            (
                "rotation_reduction_coefficient",
                GLOBAL_CONFIG.rotation_reduction_coefficient,
            ),
            ("car_accelerator", GLOBAL_CONFIG.car_accelerator),
            ("car_brake", GLOBAL_CONFIG.car_brake),
            ("car_spin", GLOBAL_CONFIG.car_spin),
            ("max_car_speed", GLOBAL_CONFIG.max_car_speed),
            ("max_car_spin", GLOBAL_CONFIG.max_car_spin),
            ("mass", 10.0),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
        .collect()
    }

    pub fn scale(&self) -> DVec3 {
        match self {
            _ => DVec3::new(1.0, 2.0, 1.0),
        }
    }

    pub fn stat(&self, stat_name: &str) -> f64 {
        match self {
            Chair::Swivel => match stat_name {
                _ => *Chair::default_stats().get(stat_name).unwrap(),
            },
            Chair::Recliner => match stat_name {
                "car_spin" => GLOBAL_CONFIG.car_spin / 15.0,
                "max_car_spin" => GLOBAL_CONFIG.max_car_spin / 2.0,
                "max_car_speed" => GLOBAL_CONFIG.max_car_spin * 5.0,
                "car_accelerator" => GLOBAL_CONFIG.car_accelerator / 15.0,
                "drag_coeffecient" => GLOBAL_CONFIG.drag_coefficient / 50.0,
                "rolling_resistance_coefficient" => {
                    GLOBAL_CONFIG.rolling_resistance_coefficient / 5.0
                }
                "mass" => 50.0,
                _ => *Chair::default_stats().get(stat_name).unwrap(),
            },
            Chair::Ergonomic => match stat_name {
                _ => *Chair::default_stats().get(stat_name).unwrap(),
            },
            Chair::Beanbag => match stat_name {
                _ => *Chair::default_stats().get(stat_name).unwrap(),
            },
            Chair::Folding => match stat_name {
                _ => *Chair::default_stats().get(stat_name).unwrap(),
            },
        }
    }
}

impl fmt::Display for Chair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            Chair::Swivel => "SuperSwivelSpinner3000",
            Chair::Recliner => "Relax-a-tron!",
            Chair::Ergonomic => "Spine Saver",
            Chair::Beanbag => "Bag-O-Beans",
            Chair::Folding => "HOW????",
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
