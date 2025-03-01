use super::toml::Config;
use crate::{
    app::{AppError, AppResult},
    widgets::logs::{add_log, LogKind},
};

pub mod find;
pub mod lint;
pub mod test;
pub use find::*;

use serde_json::to_string_pretty;

use mlua::{Error, Function, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
};

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct PluginDetails {
    pub id: String,
    pub extensions: Vec<String>,
    pub version: String,
    pub author: String,
}

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Plugin {
    pub details: PluginDetails,
    pub path: PathBuf,
    pub kind: PluginKind,
}

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum PluginKind {
    Lint,
    Test,
}

impl Plugin {
    pub fn generate<'a>(&self, toml: &Arc<Config>) -> Result<HashMap<String, String>, String> {
        let lua = Lua::new();
        add_helper_globals(&lua);
        let common_config = lua
            .to_value(&toml.common)
            .expect("unable to convert common config to lua value");
        let plugin_config = toml
            .rules
            .get(&self.details.id)
            .expect("unable to find config for a plugin");
        let plugin_config = lua
            .to_value(plugin_config)
            .expect("unable to convert plugin config to lua value");
        let plugin_config = plugin_config
            .as_table()
            .expect("unable to convert plugin config lua value to table");

        plugin_config
            .set("common", common_config)
            .expect("unable to set common table to config table");

        let generate: Result<Function, Error> = {
            let contents = match std::fs::read_to_string(&self.path.join("generate.lua")) {
                Ok(contents) => contents,
                Err(_) => {
                    return Err("Error reading plugin code".into());
                }
            };

            lua.load(contents)
                .exec()
                .map(|_| lua.globals().get("Generate").unwrap())
        };

        let validate: Result<Function, Error> = {
            let contents = match std::fs::read_to_string(&self.path.join("validate.lua")) {
                Ok(contents) => contents,
                Err(_) => {
                    return Err("Error reading plugin code".into());
                }
            };

            lua.load(contents)
                .exec()
                .map(|_| lua.globals().get("Validate").unwrap())
        };

        let validate_success = validate
            .expect("error reading validate.lua")
            .call::<mlua::Value>(plugin_config)
            .expect("error running validate function");

        let validate_success: bool = lua
            .from_value(validate_success)
            .expect("unable to convert validation result to boolean");

        if !validate_success {
            return Err("Plugin config validation failed".into());
        }

        let generate_results = generate
            .expect("Error reading generate.lua")
            .call::<mlua::Value>(plugin_config)
            .expect("error running generate function");
        let generate_results: HashMap<String, String> = lua
            .from_value(generate_results)
            .expect("unable to convert generation result to String");

        Ok(generate_results)
    }

    pub fn run<'a>(&self, toml: &Arc<Config>) -> AppResult<Vec<String>> {
        let lua = Lua::new();
        add_helper_globals(&lua);
        let common_config = lua
            .to_value(&toml.common)
            .expect("unable to convert common config to lua value");

        let plugin_config = match self.kind {
            PluginKind::Lint => toml.rules.get(&self.details.id),
            PluginKind::Test => toml.tests.get(&self.details.id),
        }
        .expect(&format!(
            "unable to find config for a plugin for {}",
            &self.details.id
        ));

        let plugin_config = lua
            .to_value(plugin_config)
            .expect("unable to convert plugin config to lua value");
        let plugin_config = plugin_config
            .as_table()
            .expect("unable to convert plugin config lua value to table");

        plugin_config
            .set("common", common_config)
            .expect("unable to set common table to config table");

        let run: Result<Function, Error> = {
            let contents = std::fs::read_to_string(&self.path.join("run.lua"))
                .expect("Error reading plugin code");

            lua.load(contents)
                .exec()
                .map(|_| lua.globals().get("Run").unwrap())
        };

        let run_success = run
            .expect("error reading run.lua")
            .call::<mlua::Value>(plugin_config)
            .expect("error running run function");

        let run_command: Vec<String> = lua
            .from_value(run_success)
            .expect("unable to parse run command");

        Ok(run_command)
    }

    pub fn list_from_config(config: &Config) -> Vec<&Plugin> {
        let linter_ids = config.rules.keys().collect::<HashSet<&String>>();
        let plugins = find::list().unwrap();

        plugins
            .iter()
            .filter(|plugin| linter_ids.contains(&plugin.details.id))
            .collect()
    }
}

fn add_helper_globals(lua: &Lua) {
    let log = lua.create_table().unwrap();
    let create_log_fn = |kind: LogKind| {
        lua.create_function(move |_, message: String| {
            add_log(kind, message);
            Ok(())
        })
        .unwrap()
    };

    let debug_print = lua
        .create_function(|_, value: Value| match to_string_pretty(&value) {
            Ok(json) => {
                add_log(LogKind::Debug, json);
                Ok(())
            }
            Err(err) => Err(mlua::Error::external(err)),
        })
        .unwrap();

    let to_json = lua
        .create_function(|_, value: Value| match to_string_pretty(&value) {
            Ok(json) => Ok(json),
            Err(err) => Err(mlua::Error::external(err)),
        })
        .unwrap();
    let to_toml = lua
        .create_function(|_, value: Value| match toml::to_string_pretty(&value) {
            Ok(toml_str) => Ok(toml_str),
            Err(err) => Err(mlua::Error::external(err)),
        })
        .unwrap();

    log.set("info", create_log_fn(LogKind::Info)).unwrap();
    log.set("error", create_log_fn(LogKind::Error)).unwrap();
    log.set("warn", create_log_fn(LogKind::Warn)).unwrap();
    log.set("success", create_log_fn(LogKind::Success)).unwrap();
    log.set("debug", debug_print).unwrap();
    lua.globals().set("to_json", to_json).unwrap();
    lua.globals().set("to_toml", to_toml).unwrap();
    lua.globals().set("log", log).unwrap();
}
