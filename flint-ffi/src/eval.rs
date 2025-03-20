use mlua::{Lua, Result as LuaResult, Table};

pub fn eval_helpers(lua: &Lua) -> LuaResult<Table> {
    let eval_table = lua.create_table()?;

    eval_table.set("lint", "__test_type_lint")?;
    eval_table.set("test", "__test_type_test")?;

    let tbl = eval_table.clone();

    eval_table.set(
        "get_output_type",
        lua.create_function(move |_, table: Table| -> LuaResult<mlua::Value> {
            if table.contains_key("Lint")? {
                Ok(tbl.get("lint")?)
            } else if table.contains_key("Test")? {
                Ok(tbl.get("test")?)
            } else {
                Ok(mlua::Value::Nil)
            }
        })?,
    )?;

    eval_table.set(
        "get_output",
        lua.create_function(move |_, table: Table| -> LuaResult<mlua::Value> {
            if table.contains_key("Lint")? {
                Ok(table.get("Lint")?)
            } else if table.contains_key("Test")? {
                Ok(table.get("Test")?)
            } else {
                Ok(mlua::Value::Nil)
            }
        })?,
    )?;

    Ok(eval_table)
}
