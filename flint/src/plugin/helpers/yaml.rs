use mlua::{Lua, Table, Value};

use crate::app::AppResult;

pub fn yaml_helpers(lua: &Lua) -> AppResult<Table> {
    let yaml = lua.create_table()?;

    let yaml_stringify =
        lua.create_function(|_, value: Value| match serde_yaml::to_string(&value) {
            Ok(yaml_str) => Ok(yaml_str),
            Err(err) => Err(mlua::Error::external(err)),
        })?;

    let yaml_parse = lua.create_function(|lua, yaml_str: String| {
        match serde_yaml::from_str::<serde_yaml::Value>(&yaml_str) {
            Ok(value) => yaml_value_to_lua_value(lua, &value),
            Err(err) => Err(mlua::Error::external(err)),
        }
    })?;

    yaml.set("stringify", yaml_stringify)?;
    yaml.set("parse", yaml_parse)?;

    Ok(yaml)
}

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
