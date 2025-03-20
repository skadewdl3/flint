use mlua::LuaSerdeExt;
use mlua::{Lua, Table, Value::Nil};

pub fn env_helpers(lua: &Lua) -> mlua::Result<Table> {
    let tbl = lua.create_table().unwrap();
    tbl.set(
        "var",
        lua.create_function(|_, name: String| -> mlua::Result<String> {
            let env_var = flint_utils::env::get_env_var(&name)?;
            Ok(env_var)
        })?,
    )?;

    tbl.set(
        "var_unsafe",
        lua.create_function(|lua, name: String| -> mlua::Result<mlua::Value> {
            Ok(match flint_utils::env::get_env_var(&name) {
                Ok(env_var) => lua.to_value(&env_var).unwrap(),
                Err(_) => mlua::Value::Nil,
            })
        })?,
    )?;

    tbl.set(
        "var_name",
        lua.create_function(|_, name: String| -> mlua::Result<String> {
            let parts = name.split(":").collect::<Vec<&str>>();
            if parts.len() != 2 {
                return Err(mlua::Error::RuntimeError("Invalid format".to_string()));
            }
            let (_, value) = (parts.get(0).unwrap(), parts.get(1).unwrap());
            Ok(value.to_string())
        })?,
    )?;

    Ok(tbl)
}
