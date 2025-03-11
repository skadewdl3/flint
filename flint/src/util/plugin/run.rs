use super::{helpers::add_helper_globals, Plugin};
use crate::{app::AppResult, util::toml::Config};
use mlua::{Error, Function, Lua, LuaSerdeExt};
use std::sync::Arc;

pub fn run<'a>(plugin: &Plugin, toml: &Arc<Config>) -> AppResult<Vec<String>> {
    let lua = Lua::new();
    add_helper_globals(&lua);
    let plugin_config = plugin.get_config_lua(&lua, toml);

    let run: Result<Function, Error> = {
        let contents = std::fs::read_to_string(plugin.path.join("run.lua"))
            .expect("Error reading plugin code");

        lua.load(contents)
            .exec()
            .map(|_| lua.globals().get("Run").unwrap())
    };

    let run_success = run
        .expect("error reading run.lua")
        .call::<mlua::Value>(plugin_config)
        .expect("error running run function");

    let run_command: Vec<String> = lua
        .from_value(run_success)
        .expect("unable to parse run command");

    Ok(run_command)
}
