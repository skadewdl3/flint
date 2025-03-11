use super::{deps::collect_dependencies, helpers::add_helper_globals, Plugin, PluginKind};
use crate::{app::AppResult, app_err, util::toml::Config};
use mlua::{Error, Function, Lua, LuaSerdeExt};
use std::{collections::HashMap, sync::Arc};

pub fn generate<'a>(plugin: &Plugin, toml: &Arc<Config>) -> AppResult<HashMap<String, String>> {
    let lua = Lua::new();
    add_helper_globals(&lua)?;

    let plugin_config = plugin.get_config_lua(&lua, toml);

    let generate: Result<Function, Error> = {
        let contents = std::fs::read_to_string(plugin.path.join("generate.lua"))?;

        lua.load(contents)
            .exec()
            .map(|_| lua.globals().get("Generate").unwrap())
    };

    let validate: Result<Function, Error> = {
        let contents = std::fs::read_to_string(plugin.path.join("validate.lua"))?;
        lua.load(contents)
            .exec()
            .map(|_| lua.globals().get("Validate").unwrap())
    };

    let validate_success = validate
        .expect("error reading validate.lua")
        .call::<mlua::Value>(&plugin_config)
        .expect("error running validate function");

    let validate_success: bool = lua
        .from_value(validate_success)
        .expect("unable to convert validation result to boolean");

    if !validate_success {
        return Err(app_err!("Plugin configuration validation failed"));
    }

    let generate_results = if plugin.kind == PluginKind::Ci {
        let active_plugins = crate::util::plugin::list_from_config(&toml);

        let dependencies = collect_dependencies(&active_plugins)?;

        let deps_table = lua.to_value(&dependencies);

        generate
            .expect("Error reading generate.lua")
            .call::<mlua::Value>((plugin_config, deps_table))
    } else {
        generate
            .expect("Error reading generate.lua")
            .call::<mlua::Value>(plugin_config)
    }
    .expect("error running generate function");

    let generate_results: HashMap<String, String> = lua
        .from_value(generate_results)
        .expect("unable to convert generation result to String");

    Ok(generate_results)
}
