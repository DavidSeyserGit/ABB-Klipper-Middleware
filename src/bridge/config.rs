use std::fs;
use std::error::Error;

#[derive(serde::Deserialize)]
pub struct Config {
    pub listener_ip: String,
    pub auth_token: String,
    pub whitelist: Option<Vec<String>>,
}

pub fn load_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let config_str = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}
