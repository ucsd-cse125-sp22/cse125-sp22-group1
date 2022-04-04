use config::{Config, ConfigError, File};
use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub port: String,
    pub server_address: String,
}

impl Settings {
    fn new() -> Result<Settings, ConfigError> {
        let config = Config::builder()
            .set_default("port", "63686")?
            .set_default("server_address", "127.0.0.1")?
            .add_source(File::with_name("config.yaml"))
            .build()?;

        config.try_deserialize()
    }
}

lazy_static! {
    pub static ref GLOBAL_CONFIG: Settings = Settings::new().expect("failed to read config file");
}