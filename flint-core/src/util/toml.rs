use crate::app::AppResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use toml;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FlintConfig {
    pub version: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub flint: FlintConfig,
    pub common: HashMap<String, toml::Value>,
    pub linters: HashMap<String, toml::Value>,
}

pub fn create_toml_config(path: &str, config: Config) -> AppResult<()> {
    let toml_str = toml::to_string(&config)?;
    std::fs::write(path, toml_str)?;
    Ok(())
}

pub fn read_toml_config(path: &str) -> AppResult<Config> {
    let toml_str = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&toml_str)?;
    Ok(config)
}
