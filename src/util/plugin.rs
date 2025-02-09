use std::{collections::BTreeSet, path::PathBuf};

use mlua::{Function, Lua, LuaSerdeExt};
use serde::{Deserialize, Serialize};

use crate::widgets::AppStatus;

use super::toml::Config;

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct PluginDetails {
    pub id: String,
    pub languages: Vec<String>,
    pub version: String,
    pub author: String,
}

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Plugin {
    pub details: PluginDetails,
    pub path: PathBuf,
}

pub fn list_plugins() -> BTreeSet<Plugin> {
    let lua = Lua::new();

    let mut plugins = BTreeSet::new();
    if let Ok(entries) = std::fs::read_dir("src/plugins") {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_path = entry.path();
                let contents = match std::fs::read_to_string(&file_path) {
                    Ok(contents) => contents,
                    Err(err) => {
                        eprintln!("Error reading file {}: {}", file_path.display(), err);
                        continue;
                    }
                };

                match lua.load(contents).exec() {
                    Ok(_) => {
                        let details: Function = lua.globals().get("Details").unwrap();
                        let lua_val = details.call::<mlua::Value>(()).unwrap();
                        let details: PluginDetails = lua.from_value(lua_val).unwrap();
                        plugins.insert(Plugin {
                            details,
                            path: file_path,
                        });
                    }
                    Err(err) => {
                        eprintln!("Error loading lua file {}: {}", file_path.display(), err);
                        continue;
                    }
                }
            }
        }
    }
    plugins
}

pub fn run_plugin<'a>(toml: &Config, plugin: &Plugin) -> Result<String, &'a str> {
    let lua = Lua::new();
    let plugin_config = toml
        .linters
        .get(&plugin.details.id)
        .expect("unable to find config for a plugin");
    let plugin_config = lua
        .to_value(plugin_config)
        .expect("unable to convert plugin config to lua value");
    let plugin_config = plugin_config
        .as_table()
        .expect("unable to convert plugin config lua value to table");

    let contents = match std::fs::read_to_string(&plugin.path) {
        Ok(contents) => contents,
        Err(_) => {
            return Err("Error reading plugin code");
        }
    };

    let (validate, generate) = match lua.load(contents).exec() {
        Ok(_) => {
            let validate: Function = lua
                .globals()
                .get("Validate")
                .expect("could not find validate function in plugin file");
            let generate: Function = lua
                .globals()
                .get("Generate")
                .expect("could not find generate function in plugin file");
            (validate, generate)
        }
        Err(_) => {
            return Err("Error loading lua file");
        }
    };

    let validate_success = validate
        .call::<mlua::Value>(plugin_config)
        .expect("error running validate function");

    let validate_success: bool = lua
        .from_value(validate_success)
        .expect("unable to convert validation result to boolean");
    if !validate_success {
        return Err("Plugin config validation failed");
    }

    let generate_results = generate
        .call::<mlua::Value>(plugin_config)
        .expect("error running generate function");
    let generate_results: String = lua
        .from_value(generate_results)
        .expect("unable to convert generation result to String");

    Ok(generate_results)
}
