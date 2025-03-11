use mlua::{Lua, Table, Value};

use crate::app::AppResult;

pub fn toml_helpers(lua: &Lua) -> AppResult<Table> {
    let toml = lua.create_table()?;

    let toml_stringify =
        lua.create_function(|_, value: Value| match toml::to_string_pretty(&value) {
            Ok(toml_str) => Ok(toml_str),
            Err(err) => Err(mlua::Error::external(err)),
        })?;

    let toml_parse = lua.create_function(|lua, toml_str: String| {
        match toml::from_str::<toml::Value>(&toml_str) {
            Ok(value) => toml_value_to_lua_value(lua, &value),
            Err(err) => Err(mlua::Error::external(err)),
        }
    })?;

    toml.set("stringify", toml_stringify)?;
    toml.set("parse", toml_parse)?;

    Ok(toml)
}

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
