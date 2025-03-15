use mlua::{Lua, Result as LuaResult, Table, Value};
use serde_json::to_string_pretty;

pub fn json_helpers(lua: &Lua) -> LuaResult<Table> {
    let json = lua.create_table()?;

    let json_stringify = lua.create_function(|_, value: Value| match to_string_pretty(&value) {
        Ok(json) => Ok(json),
        Err(err) => Err(mlua::Error::external(err)),
    })?;

    let json_parse = lua.create_function(|lua, json_str: String| {
        match serde_json::from_str::<serde_json::Value>(&json_str) {
            Ok(value) => serde_value_to_lua_value(lua, &value),
            Err(err) => Err(mlua::Error::external(err)),
        }
    })?;

    json.set("stringify", json_stringify)?;
    json.set("parse", json_parse)?;

    Ok(json)
}

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
