use serde::Serialize;
use toml;

#[derive(Serialize)]
struct Config {
    ip: String,
    port: Option<u16>,
    keys: Keys,
}

#[derive(Serialize)]
struct Keys {
    github: String,
    travis: Option<String>,
}

pub fn create_toml_config(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        ip: "127.0.0.1".to_string(),
        port: None,
        keys: Keys {
            github: "xxxxxxxxxxxxxxxxx".to_string(),
            travis: Some("yyyyyyyyyyyyyyyyy".to_string()),
        },
    };

    let toml_str = toml::to_string(&config).unwrap();
    std::fs::write(path, toml_str)?;

    Ok(())
}
