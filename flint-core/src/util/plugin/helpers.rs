use mlua::{Lua, Value};
use serde_json::to_string_pretty;

use crate::widgets::logs::{add_log, LogKind};

pub fn add_helper_globals(lua: &Lua) {
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
                // Handle different key types for YAML mappings
                match k {
                    serde_yaml::Value::String(key_str) => {
                        lua_table.set(key_str.clone(), yaml_value_to_lua_value(lua, v)?)?;
                    }
                    serde_yaml::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            lua_table.set(i, yaml_value_to_lua_value(lua, v)?)?;
                        } else if let Some(f) = n.as_f64() {
                            // Convert float keys to strings
                            let key_str = lua.create_string(&f.to_string())?;
                            lua_table.set(key_str, yaml_value_to_lua_value(lua, v)?)?;
                        }
                    }
                    serde_yaml::Value::Bool(b) => {
                        // Convert boolean keys to strings
                        let key_str = lua.create_string(if *b { "true" } else { "false" })?;
                        lua_table.set(key_str, yaml_value_to_lua_value(lua, v)?)?;
                    }
                    _ => {
                        // For complex keys, serialize them to YAML and use that as the key
                        // This is a fallback for complex types
                        match serde_yaml::to_string(k) {
                            Ok(key_str) => {
                                // Trim the trailing newline that serde_yaml adds
                                let key_str = key_str.trim_end();
                                let lua_str = lua.create_string(key_str)?;
                                lua_table.set(lua_str, yaml_value_to_lua_value(lua, v)?)?;
                            }
                            Err(_) => {
                                // If serialization fails, use a placeholder
                                let key_str = lua.create_string("complex_key")?;
                                lua_table.set(key_str, yaml_value_to_lua_value(lua, v)?)?;
                            }
                        }
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
