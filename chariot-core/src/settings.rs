use config::{Config, ConfigError, File};
use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub port: String,
    pub server_address: String,
    pub server_tick_ms: u64,
    pub player_amount: usize
}

impl Settings {
    fn new() -> Result<Settings, ConfigError> {
        let config = Config::builder()
            .set_default("port", "24247")?
            .set_default("server_address", "127.0.0.1")?
            .set_default("server_tick_ms", 30)?
            .set_default("player_amount", 1)?
            .add_source(File::with_name("config.yaml").required(false))
            .build()?;

        config.try_deserialize()
    }
}

lazy_static! {
    pub static ref GLOBAL_CONFIG: Settings = Settings::new().expect("failed to read config file");
}