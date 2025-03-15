use flint_utils::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use toml;

fn default_plugins_branch() -> String {
    "main".into()
}

fn default_hashmap() -> HashMap<String, toml::Value> {
    HashMap::new()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FlintConfig {
    pub version: u8,
    #[serde(default = "default_plugins_branch")]
    pub plugins_branch: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub flint: FlintConfig,

    #[serde(default = "default_hashmap")]
    pub rules: HashMap<String, toml::Value>,
    #[serde(default = "default_hashmap")]
    pub tests: HashMap<String, toml::Value>,

    #[serde(default = "default_hashmap")]
    pub config: HashMap<String, toml::Value>,

    #[serde(default = "default_hashmap")]
    pub ci: HashMap<String, toml::Value>,

    #[serde(default = "default_hashmap")]
    pub report: HashMap<String, toml::Value>,
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self> {
        let toml_str = std::fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&toml_str)?;
        Ok(config)
    }

    pub fn create(path: PathBuf, config: Config) -> Result<()> {
        let toml_str = toml::to_string(&config)?;
        std::fs::write(path, toml_str)?;
        Ok(())
    }

    pub fn create_default(path: PathBuf) -> Result<()> {
        let config = Config {
            flint: FlintConfig {
                version: 1,
                plugins_branch: "main".into(),
            },
            rules: HashMap::new(),
            tests: HashMap::new(),
            config: HashMap::new(),
            ci: HashMap::new(),
            report: HashMap::new(),
        };
        Self::create(path, config)
    }
}
