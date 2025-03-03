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
    let json = lua.create_table().unwrap();
    let toml = lua.create_table().unwrap();
    let yaml = lua.create_table().unwrap();

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

    let json_stringify = lua
        .create_function(|_, value: Value| match to_string_pretty(&value) {
            Ok(json) => Ok(json),
            Err(err) => Err(mlua::Error::external(err)),
        })
        .unwrap();
    let toml_stringify = lua
        .create_function(|_, value: Value| match toml::to_string_pretty(&value) {
            Ok(toml_str) => Ok(toml_str),
            Err(err) => Err(mlua::Error::external(err)),
        })
        .unwrap();

    let yaml_stringify = lua
        .create_function(|_, value: Value| match serde_yaml::to_string(&value) {
            Ok(yaml_str) => Ok(yaml_str),
            Err(err) => Err(mlua::Error::external(err)),
        })
        .unwrap();

    let json_parse = lua
        .create_function(|lua, json_str: String| {
            match serde_json::from_str::<serde_json::Value>(&json_str) {
                Ok(value) => serde_value_to_lua_value(lua, &value),
                Err(err) => Err(mlua::Error::external(err)),
            }
        })
        .unwrap();

    // Create the TOML parse function
    let toml_parse = lua
        .create_function(
            |lua, toml_str: String| match toml::from_str::<toml::Value>(&toml_str) {
                Ok(value) => toml_value_to_lua_value(lua, &value),
                Err(err) => Err(mlua::Error::external(err)),
            },
        )
        .unwrap();

    // Create the YAML parse function
    let yaml_parse = lua
        .create_function(|lua, yaml_str: String| {
            match serde_yaml::from_str::<serde_yaml::Value>(&yaml_str) {
                Ok(value) => yaml_value_to_lua_value(lua, &value),
                Err(err) => Err(mlua::Error::external(err)),
            }
        })
        .unwrap();

    log.set("info", create_log_fn(LogKind::Info)).unwrap();
    log.set("error", create_log_fn(LogKind::Error)).unwrap();
    log.set("warn", create_log_fn(LogKind::Warn)).unwrap();
    log.set("success", create_log_fn(LogKind::Success)).unwrap();
    log.set("debug", debug_print).unwrap();

    json.set("stringify", json_stringify).unwrap();
    json.set("parse", json_parse).unwrap();

    toml.set("stringify", toml_stringify).unwrap();
    toml.set("parse", toml_parse).unwrap();

    yaml.set("stringify", yaml_stringify).unwrap();
    yaml.set("parse", yaml_parse).unwrap();

    lua.globals().set("log", log).unwrap();
    lua.globals().set("json", json).unwrap();
    lua.globals().set("toml", toml).unwrap();
    lua.globals().set("yaml", yaml).unwrap();
}

// Helper function to convert serde_json::Value to mlua::Value

// JSON value to Lua value converter
fn serde_value_to_lua_value<'lua>(
    lua: &'lua Lua,
    value: &serde_json::Value,
) -> Result<Value, mlua::Error> {
    match value {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(b) => Ok(Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i as i32))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Err(mlua::Error::external("Unsupported number format"))
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(lua.create_string(s)?)),
        serde_json::Value::Array(arr) => {
            let lua_table = lua.create_table()?;
            for (i, val) in arr.iter().enumerate() {
                lua_table.set(i + 1, serde_value_to_lua_value(lua, val)?)?;
            }
            Ok(Value::Table(lua_table))
        }
        serde_json::Value::Object(obj) => {
            let lua_table = lua.create_table()?;
            for (k, v) in obj.iter() {
                lua_table.set(k.clone(), serde_value_to_lua_value(lua, v)?)?;
            }
            Ok(Value::Table(lua_table))
        }
    }
}

// TOML value to Lua value converter
fn toml_value_to_lua_value<'lua>(
    lua: &'lua Lua,
    value: &toml::Value,
) -> Result<Value, mlua::Error> {
    match value {
        toml::Value::String(s) => Ok(Value::String(lua.create_string(s)?)),
        toml::Value::Integer(i) => Ok(Value::Integer(*i as i32)),
        toml::Value::Float(f) => Ok(Value::Number(*f)),
        toml::Value::Boolean(b) => Ok(Value::Boolean(*b)),
        toml::Value::Datetime(dt) => Ok(Value::String(lua.create_string(&dt.to_string())?)),
        toml::Value::Array(arr) => {
            let lua_table = lua.create_table()?;
            for (i, val) in arr.iter().enumerate() {
                lua_table.set(i + 1, toml_value_to_lua_value(lua, val)?)?;
            }
            Ok(Value::Table(lua_table))
        }
        toml::Value::Table(tbl) => {
            let lua_table = lua.create_table()?;
            for (k, v) in tbl.iter() {
                lua_table.set(k.clone(), toml_value_to_lua_value(lua, v)?)?;
            }
            Ok(Value::Table(lua_table))
        }
    }
}

// YAML value to Lua value converter
fn yaml_value_to_lua_value<'lua>(
    lua: &'lua Lua,
    value: &serde_yaml::Value,
) -> Result<Value, mlua::Error> {
    match value {
        serde_yaml::Value::Null => Ok(Value::Nil),
        serde_yaml::Value::Bool(b) => Ok(Value::Boolean(*b)),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i as i32))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Err(mlua::Error::external("Unsupported number format"))
            }
        }
        serde_yaml::Value::String(s) => Ok(Value::String(lua.create_string(s)?)),
        serde_yaml::Value::Sequence(seq) => {
            let lua_table = lua.create_table()?;
            for (i, val) in seq.iter().enumerate() {
                lua_table.set(i + 1, yaml_value_to_lua_value(lua, val)?)?;
            }
            Ok(Value::Table(lua_table))
        }
        serde_yaml::Value::Mapping(map) => {
            let lua_table = lua.create_table()?;
            for (k, v) in map.iter() {
                // Keys in YAML can be complex types, so we need to convert them to strings
                // for Lua table keys in some cases
                match yaml_value_to_lua_value(lua, k)? {
                    Value::String(key_str) => {
                        lua_table.set(key_str, yaml_value_to_lua_value(lua, v)?)?;
                    }
                    Value::Integer(key_int) => {
                        lua_table.set(key_int, yaml_value_to_lua_value(lua, v)?)?;
                    }
                    _ => {
                        // For complex keys, convert to string representation
                        let key_str = lua.create_string(&k.to_string())?;
                        lua_table.set(key_str, yaml_value_to_lua_value(lua, v)?)?;
                    }
                }
            }
            Ok(Value::Table(lua_table))
        }
        serde_yaml::Value::Tagged(tagged) => {
            // Handle tagged values (just use the value)
            yaml_value_to_lua_value(lua, &tagged.value)
        }
    }
}
