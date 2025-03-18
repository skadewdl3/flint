use std::sync::Arc;

use mlua::{Function, Lua, Result as LuaResult, Table, Value, Variadic};
use tokio::runtime::Runtime;

pub fn coroutine_helpers(lua: &Lua, rt: Arc<Runtime>) -> LuaResult<Table> {
    let await_fn = lua.create_function(move |_, args: Variadic<Value>| {
        if args.is_empty() {
            return Err(mlua::Error::runtime(
                "Expected at least one argument (function)",
            ));
        }

        let func: Function = match args.get(0) {
            Some(Value::Function(f)) => f.clone(),
            _ => return Err(mlua::Error::runtime("First argument must be a function")),
        };

        let callback_args = args.iter().skip(1).cloned().collect::<Vec<_>>();

        // Use the provided Tokio runtime instead of creating a new one
        let result = rt.block_on(async {
            func.call_async::<mlua::Value>(Variadic::from_iter(callback_args))
                .await
        });

        result
    })?;

    let async_table = lua.create_table()?;
    async_table.set("await", await_fn)?;
    Ok(async_table)
}
