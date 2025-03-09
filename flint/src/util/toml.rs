use crate::app::AppResult;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use toml;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FlintConfig {
    pub version: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub flint: FlintConfig,
    pub common: HashMap<String, toml::Value>,
    pub rules: HashMap<String, toml::Value>,
    pub tests: HashMap<String, toml::Value>,
    pub config: HashMap<String, toml::Value>,
    pub ci: HashMap<String, toml::Value>,
    pub report: HashMap<String, toml::Value>,
}

impl Config {
    pub fn load(path: PathBuf) -> AppResult<Self> {
        let toml_str = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&toml_str)?;
        Ok(config)
    }

    pub fn create(path: PathBuf, config: Config) -> AppResult<()> {
        let toml_str = toml::to_string(&config)?;
        std::fs::write(path, toml_str)?;
        Ok(())
    }

    pub fn create_default(path: PathBuf) -> AppResult<()> {
        let config = Config {
            flint: FlintConfig { version: 1 },
            common: HashMap::new(),
            rules: HashMap::new(),
            tests: HashMap::new(),
            config: HashMap::new(),
            ci: HashMap::new(),
            report: HashMap::new(),
        };
        Self::create(path, config)
    }
}
