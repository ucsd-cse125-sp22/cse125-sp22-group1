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

	// Physics
	pub gravity_coefficient: f64,
	pub drag_coefficient: f64,
	pub rolling_resistance_coefficient: f64,

	pub rotation_reduction_coefficient: f64,

	pub car_accelerator: f64,
	pub car_brake: f64,
	pub car_spin: f64,
}

impl Settings {
    fn new() -> Result<Settings, ConfigError> {
        let config = Config::builder()
			// networking
            .set_default("port", "24247")?
            .set_default("server_address", "127.0.0.1")?
            .set_default("server_tick_ms", 30)?
            .set_default("player_amount", 1)?
			// physics
			.set_default("gravity_coefficient", 1.0)?
			.set_default("drag_coefficient", 0.01)?
			.set_default("rolling_resistance_coefficient", 0.3)?
			.set_default("rotation_reduction_coefficient", 0.95)?
			.set_default("car_accelerator", 1.0)?
			.set_default("car_brake", 0.1)?
			.set_default("car_spin", 0.1)?
            .add_source(File::with_name("config.yaml").required(false))
            .build()?;

        config.try_deserialize()
    }
}

lazy_static! {
    pub static ref GLOBAL_CONFIG: Settings = Settings::new().expect("failed to read config file");
}