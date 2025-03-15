use mlua::{Lua, Result as LuaResult, Table};

pub fn ai_helpers(lua: &Lua) -> LuaResult<Table> {
    let table = lua.create_table()?;
    Ok(table)
}
