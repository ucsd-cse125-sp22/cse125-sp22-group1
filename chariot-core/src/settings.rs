use config::{Config, ConfigError, File};
use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    // Networking
    pub port: String,
    pub server_address: String,
    pub server_tick_ms: u64,
    pub player_amount: usize,
    pub ws_server_port: String,

    // Physics
    pub gravity_coefficient: f64,
    pub drag_coefficient: f64,
    pub rolling_resistance_coefficient: f64,

    pub rotation_reduction_coefficient: f64,

    pub car_accelerator: f64,
    pub car_brake: f64,
    pub car_spin: f64,

    pub max_car_speed: f64,
    pub max_car_spin: f64,

    pub audience_vote_time_ms: u64,
}

impl Settings {
    fn new() -> Result<Settings, ConfigError> {
        let config = Config::builder()
            // networking
            .set_default("port", "24247")?
            .set_default("server_address", "127.0.0.1")?
            .set_default("ws_server_port", "2334")?
            .set_default("server_tick_ms", 30)?
            .set_default("player_amount", 1)?
            // physics
            // `gravity_coefficient` is acceleration due to gravity: this is
            // little g (whose IRL value is 9.81 meters per second squared, but
            // we are not operating in those units so this is a placeholder
            // value for now).
            .set_default("gravity_coefficient", 1.0)?
            // We model air resistance with a (very) simplified model of
            // `drag_coefficient` times velocity squared. Since drag is
            // quadratic and friction is linear, this coefficient should be much
            // smaller (~30 times smaller is realistic) than
            // `rolling_resistance_coefficient` to have the (correct) property
            // that drag dominates at higher speeds.
            .set_default("drag_coefficient", 0.003)?
            // Rolling resistance is modelled as being linearly proportional to
            // velocity; see notes about the drag coefficient for information
            // about their relative magnitudes.
            .set_default("rolling_resistance_coefficient", 0.08)?
            // This doesn't have a real-world equivalent, but we might call it
            // the rotational analogue of friction: each time step in free
            // rotation, what proportion of angular velocity should be
            // conserved?
            .set_default("rotation_reduction_coefficient", 0.80)?
            // Coefficient to control how forceful player-controlled acceleration is
            .set_default("car_accelerator", 0.8)?
            // Coefficient to control how forceful player-controlled braking is
            .set_default("car_brake", 0.05)?
            // Coefficient to control how fast the player can spin
            .set_default("car_spin", 0.025)?
            .set_default("max_car_speed", 1.0)?
            .set_default("max_car_spin", 0.25)?
            .set_default("audience_vote_time_ms", 30000)?
            .add_source(File::with_name("config.yaml").required(false))
            .build()?;

        config.try_deserialize()
    }
}

lazy_static! {
    pub static ref GLOBAL_CONFIG: Settings = Settings::new().expect("failed to read config file");
}
