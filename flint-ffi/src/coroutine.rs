use mlua::{Function, Lua, Result as LuaResult, Table, Value, Variadic, chunk};

pub fn coroutine_helpers(lua: &Lua) -> LuaResult<Table> {
    // Create a function that takes an async mlua function and blocks until it completes

    let block_on = lua.create_function(|_, args: Variadic<Value>| {
        if args.len() < 1 {
            return Err(mlua::Error::runtime(
                "Expected at least one argument (function)",
            ));
        }

        let func: Function = match args.get(0) {
            Some(Value::Function(f)) => f.clone(),
            _ => return Err(mlua::Error::runtime("First argument must be a function")),
        };
        let callable_func = func.clone();
        // Get all arguments after the function itself
        let callback_args = args.iter().skip(1).cloned().collect::<Vec<_>>();
        let result = smol::block_on(async {
            callable_func
                .call_async::<mlua::Value>(Variadic::from_iter(callback_args))
                .await
        });
        result
    })?;

    let async_table: Table = lua.create_table()?;

    // Function to provide await functionality for Lua async functions
    let await_fn = lua.create_function(|_, future_value: Value| {
        match future_value {
            // If it's already a function, call it directly
            Value::Function(func) => smol::block_on(async { func.call_async::<Value>(()).await }),
            _ => Err(mlua::Error::runtime("Cannot await non-future value")),
        }
    })?;

    // Set the await function as a global
    lua.globals().set("await", await_fn)?;

    async_table.set("block_on", block_on)?;

    Ok(async_table)
}
