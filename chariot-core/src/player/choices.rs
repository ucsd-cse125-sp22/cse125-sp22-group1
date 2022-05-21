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
            chair: Chair::Swivel,
            map: Track::Track,
            ready: false,
        }
    }
}

lazy_static! {
    static ref DEFAULT_CHAIR: HashMap<String, f64> = [
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
    .collect();
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
            Chair::Ergonomic => "ergonomic",
            Chair::Beanbag => "beanbag",
            Chair::Folding => "folding",
        }
        .to_string()
    }

    fn default_stats() -> &'static HashMap<String, f64> {
        &*DEFAULT_CHAIR
    }

    pub fn scale(&self) -> DVec3 {
        match self {
            _ => DVec3::new(1.0, 2.0, 1.0),
        }
    }

    pub fn stat(&self, stat_name: &str) -> f64 {
        match self {
            Chair::Swivel => match stat_name {
                // Keep rolling for a bit
                "rolling_resistance_coefficient" => {
                    GLOBAL_CONFIG.rolling_resistance_coefficient * 0.5
                }

                "drag_coefficient" => GLOBAL_CONFIG.drag_coefficient * 0.02,
                _ => *Chair::default_stats().get(stat_name).unwrap(),
            },
            Chair::Recliner => match stat_name {
                // Make our turn very hefty
                "car_spin" => GLOBAL_CONFIG.car_spin * 0.06,
                "max_car_spin" => GLOBAL_CONFIG.max_car_spin * 0.7,
                // Can get going fast
                "max_car_speed" => GLOBAL_CONFIG.max_car_speed * 1.1,
                // But takes a bit to get there
                "car_accelerator" => GLOBAL_CONFIG.car_accelerator * 0.75,
                // However, it will NOT stop.
                "drag_coeffecient" => GLOBAL_CONFIG.drag_coefficient * 0.02,
                "rolling_resistance_coefficient" => {
                    GLOBAL_CONFIG.rolling_resistance_coefficient * 0.2
                }
                // We have a bit of braking power, though
                "car_brake" => GLOBAL_CONFIG.car_brake * 10.0,
                "mass" => 50.0,
                _ => *Chair::default_stats().get(stat_name).unwrap(),
            },
            Chair::Ergonomic => match stat_name {
                // We can turn on a dime
                "max_car_spin" => GLOBAL_CONFIG.max_car_spin * 1.9,
                "rolling_resistance_coefficient" => {
                    GLOBAL_CONFIG.rolling_resistance_coefficient * 3.0
                }
                // But are a bit slower
                "max_car_speed" => GLOBAL_CONFIG.max_car_speed * 0.8,
                // We have great control over our direction
                "car_accelerator" => GLOBAL_CONFIG.car_accelerator * 2.0,
                // And won't roll too when changing direction
                "drag_coeffecient" => GLOBAL_CONFIG.drag_coefficient * 1.5,
                // And we have a decent break
                "car_brake" => GLOBAL_CONFIG.car_brake * 15.0,
                _ => *Chair::default_stats().get(stat_name).unwrap(),
            },
            Chair::Beanbag => match stat_name {
                "max_car_speed" => GLOBAL_CONFIG.max_car_speed * 1.8,
                "car_accelerator" => GLOBAL_CONFIG.car_accelerator * 0.33,
                // A L L G A S N O B R A K E S
                "car_brake" => 0.0,
                // very light on our feet :^)
                "drag_coefficient" => GLOBAL_CONFIG.drag_coefficient * 0.001,
                "rolling_resistance_coefficient" => {
                    GLOBAL_CONFIG.rolling_resistance_coefficient * 0.1
                }
                "mass" => 1.0,
                _ => *Chair::default_stats().get(stat_name).unwrap(),
            },
            Chair::Folding => match stat_name {
                _ => unimplemented!(
                    "I don't have a lot of ideas for this one currently. May add later!"
                ),
                //_ => *Chair::default_stats().get(stat_name).unwrap(),
            },
        }
    }

    pub fn cam(&self) -> CameraType {
        match self {
            Chair::Swivel => CameraType::FaceForwards,
            Chair::Recliner => CameraType::FaceVelocity,
            Chair::Ergonomic => CameraType::FaceForwards,
            Chair::Beanbag => CameraType::FaceForwards,
            Chair::Folding => CameraType::FaceVelocity,
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
            Chair::Folding => "Plastic Penny Pincher",
        };
        write!(f, "{}", printable)
    }
}

#[derive(Clone, Copy)]
pub enum CameraType {
    FaceForwards,
    FaceVelocity,
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
