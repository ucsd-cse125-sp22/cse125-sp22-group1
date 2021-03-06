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

    pub start_fullscreen: bool,

    // Resources
    pub tracks_folder: String,

    // Gameplay
    pub number_laps: u8,
    pub powerup_cooldown_time: u64,
    pub volume: f32,
    pub enable_particle_effects: bool,

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

    pub wall_bounciness: f64,
    pub player_bounciness: f64,

    pub off_track_speed_penalty: f64,

    // Voting
    pub audience_vote_time_ms: u64,
}

impl Settings {
    fn new() -> Result<Settings, ConfigError> {
        let config = Config::builder()
            // networking
            .set_default("port", "24247")?
            .set_default("server_address", "127.0.0.1")?
            .set_default("ws_server_port", "0.0.0.0:2334")?
            .set_default("server_tick_ms", 30)?
            .set_default("player_amount", 4)?
            // display settings
            .set_default("start_fullscreen", true)?
            // tracks folder (too big to embed)
            .set_default("tracks_folder", "../tracks")?
            // Gameplay
            .set_default("number_laps", 3)?
            .set_default("powerup_cooldown_time", 10)?
            .set_default("volume", 1.0)?
            .set_default("enable_particle_effects", true)?
            // physics
            // `gravity_coefficient` is acceleration due to gravity: this is
            // little g (whose IRL value is 9.81 meters per second squared, but
            // we are not operating in those units so this is a placeholder
            // value for now).
            .set_default("gravity_coefficient", 0.01)?
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
            .set_default("rotation_reduction_coefficient", 0.40)?
            // Coefficient to control how forceful player-controlled acceleration is
            .set_default("car_accelerator", 0.8 / 15.0)?
            // Coefficient to control how forceful player-controlled braking is
            .set_default("car_brake", 0.005)?
            // Coefficient to control how fast the player can spin
            .set_default("car_spin", 0.03)?
            .set_default("max_car_speed", 0.5)?
            .set_default("max_car_spin", 0.1)?
            // How hard we should bounce off the walls (1.0 = as fast as we were initially going)
            .set_default("wall_bounciness", 3.0)?
            // How hard we should bounce off other players (1.0 = real-world physically accurate)
            .set_default("player_bounciness", 3.0)?
            // How much slower should you go when off-track? (0.20 => 80% of on-track speed when off)
            .set_default("off_track_speed_penalty", 0.20)?
            .set_default("audience_vote_time_ms", 30000)?
            .add_source(File::with_name("config.yaml").required(false))
            .build()?;

        config.try_deserialize()
    }
}

lazy_static! {
    pub static ref GLOBAL_CONFIG: Settings = Settings::new().expect("failed to read config file");
}
