use glam::DVec3;
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    fmt::{self},
};

use serde::{Deserialize, Serialize};

use crate::GLOBAL_CONFIG;

#[derive(Clone, Serialize, Deserialize, Debug)]
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
    static ref DEFAULT_CHAIR: HashMap<Stat, f64> = [
        (Stat::GravityCoefficient, GLOBAL_CONFIG.gravity_coefficient),
        (Stat::DragCoefficient, GLOBAL_CONFIG.drag_coefficient),
        (
            Stat::RollingResistanceCoefficient,
            GLOBAL_CONFIG.rolling_resistance_coefficient,
        ),
        (
            Stat::RotationReductionCoefficient,
            GLOBAL_CONFIG.rotation_reduction_coefficient,
        ),
        (Stat::CarAccelerator, GLOBAL_CONFIG.car_accelerator),
        (Stat::CarBrake, GLOBAL_CONFIG.car_brake),
        (Stat::CarSpin, GLOBAL_CONFIG.car_spin),
        (Stat::MaxCarSpeed, GLOBAL_CONFIG.max_car_speed),
        (Stat::MaxCarSpin, GLOBAL_CONFIG.max_car_spin),
        (Stat::Mass, GLOBAL_CONFIG.gravity_coefficient),
    ]
    .iter()
    .map(|(k, v)| (*k, *v))
    .collect();
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum Stat {
    GravityCoefficient,
    DragCoefficient,
    RollingResistanceCoefficient,
    RotationReductionCoefficient,
    CarAccelerator,
    CarBrake,
    CarSpin,
    MaxCarSpeed,
    MaxCarSpin,
    Mass,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum Chair {
    Swivel,
    Recliner,
    Beanbag,
    Ergonomic,
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

    fn default_stats() -> &'static HashMap<Stat, f64> {
        &*DEFAULT_CHAIR
    }

    pub fn scale(&self) -> DVec3 {
        match self {
            _ => DVec3::new(1.0, 2.0, 1.0),
        }
    }

    pub fn stat(&self, stat_name: &Stat) -> f64 {
        match self {
            Chair::Swivel => match stat_name {
                // Keep rolling for a bit
                Stat::RollingResistanceCoefficient => {
                    GLOBAL_CONFIG.rolling_resistance_coefficient * 0.5
                }

                Stat::DragCoefficient => GLOBAL_CONFIG.drag_coefficient * 0.02,
                _ => *Chair::default_stats().get(stat_name).unwrap(),
            },
            Chair::Recliner => match stat_name {
                // Make our turn very hefty
                Stat::CarSpin => GLOBAL_CONFIG.car_spin * 0.06,
                Stat::MaxCarSpin => GLOBAL_CONFIG.max_car_spin * 0.7,
                // Can get going fast
                Stat::MaxCarSpeed => GLOBAL_CONFIG.max_car_speed * 1.1,
                // But takes a bit to get there
                Stat::CarAccelerator => GLOBAL_CONFIG.car_accelerator * 0.75,
                // However, it will NOT stop.
                Stat::DragCoefficient => GLOBAL_CONFIG.drag_coefficient * 0.02,
                Stat::RollingResistanceCoefficient => {
                    GLOBAL_CONFIG.rolling_resistance_coefficient * 0.2
                }
                // We have a bit of braking power, though
                Stat::CarBrake => GLOBAL_CONFIG.car_brake * 10.0,
                Stat::Mass => 50.0,
                _ => *Chair::default_stats().get(stat_name).unwrap(),
            },
            Chair::Ergonomic => match stat_name {
                // We can turn on a dime
                Stat::MaxCarSpin => GLOBAL_CONFIG.max_car_spin * 1.9,
                Stat::RollingResistanceCoefficient => {
                    GLOBAL_CONFIG.rolling_resistance_coefficient * 3.0
                }
                // But are a bit slower
                Stat::MaxCarSpeed => GLOBAL_CONFIG.max_car_speed * 0.8,
                // We have great control over our direction
                Stat::CarAccelerator => GLOBAL_CONFIG.car_accelerator * 2.0,
                // And won't roll too when changing direction
                Stat::DragCoefficient => GLOBAL_CONFIG.drag_coefficient * 1.5,
                // And we have a decent break
                Stat::CarBrake => GLOBAL_CONFIG.car_brake * 15.0,
                _ => *Chair::default_stats().get(stat_name).unwrap(),
            },
            Chair::Beanbag => match stat_name {
                Stat::MaxCarSpeed => GLOBAL_CONFIG.max_car_speed * 1.8,
                Stat::CarAccelerator => GLOBAL_CONFIG.car_accelerator * 0.33,
                // A L L G A S N O B R A K E S
                Stat::CarBrake => 0.0,
                // very light on our feet :^)
                Stat::DragCoefficient => GLOBAL_CONFIG.drag_coefficient * 0.001,
                Stat::RollingResistanceCoefficient => {
                    GLOBAL_CONFIG.rolling_resistance_coefficient * 0.1
                }
                Stat::Mass => 1.0,
                _ => *Chair::default_stats().get(stat_name).unwrap(),
            },
            Chair::Folding => match stat_name {
                _ => *Chair::default_stats().get(stat_name).unwrap(),
            },
        }
    }

    pub fn cam(&self) -> CameraType {
        match self {
            Chair::Swivel => CameraType::FaceForwards,
            Chair::Recliner => CameraType::FaceForwards,
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

#[derive(Clone, Serialize, Deserialize, Debug)]
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
