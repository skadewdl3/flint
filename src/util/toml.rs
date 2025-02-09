use serde::Serialize;
use toml;

#[derive(Serialize)]
pub struct FlintConfig {
    langs: Vec<String>,
}

#[derive(Serialize)]
pub struct Config {
    flint: FlintConfig,
}

pub fn create_toml_config(path: &str, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let toml_str = toml::to_string(&config).unwrap();
    std::fs::write(path, toml_str)?;

    Ok(())
}
