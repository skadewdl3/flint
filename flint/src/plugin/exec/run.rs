use crate::{plugin::Plugin, util::toml::Config};
use flint_ffi::add_ffi_modules;
use flint_utils::Result;
use mlua::{Function, Lua, LuaSerdeExt};
use std::sync::Arc;

pub fn run<'a>(plugin: &Plugin, toml: &Arc<Config>) -> Result<Vec<String>> {
    let lua = Lua::new();
    add_ffi_modules(&lua)?;
    let plugin_config = plugin.get_config_lua(&lua, toml);

    let run: Function = {
        let contents = std::fs::read_to_string(plugin.path.join("run.lua"))
            .expect("Error reading plugin code");

        lua.load(contents)
            .exec()
            .map(|_| lua.globals().get("Run").unwrap())
    }?;

    let run_success = run.call::<mlua::Value>(plugin_config)?;

    let run_command: Vec<String> = lua.from_value(run_success)?;

    Ok(run_command)
}
