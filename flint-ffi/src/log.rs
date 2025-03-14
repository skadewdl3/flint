use flint_utils::{Result, debug, error, info, success, warn};
use mlua::{Lua, Table, Value};
use serde_json::to_string_pretty;

pub fn log_helpers(lua: &Lua) -> Result<Table> {
    let log = lua.create_table()?;
    let create_log_fn = |kind| {
        lua.create_function(move |_, message: String| {
            match kind {
                "info" => info!("{}", message),
                "error" => error!("{}", message),
                "warn" => warn!("{}", message),
                "success" => success!("{}", message),
                "debug" => debug!("{}", message),
                _ => info!("{}", message),
            }
            Ok(())
        })
        .unwrap()
    };

    // Define debug_print function
    let debug_print = lua.create_function(|_, value: Value| match to_string_pretty(&value) {
        Ok(json) => {
            debug!("{}", json);
            Ok(())
        }
        Err(err) => Err(mlua::Error::external(err)),
    })?;

    log.set("info", create_log_fn("info"))?;
    log.set("error", create_log_fn("error"))?;
    log.set("warn", create_log_fn("warn"))?;
    log.set("success", create_log_fn("success"))?;
    log.set("debug", debug_print)?;
    Ok(log)
}
