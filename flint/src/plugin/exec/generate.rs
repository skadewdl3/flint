use crate::{
    plugin::{deps::collect_dependencies, Plugin, PluginKind},
    util::toml::Config,
};
use flint_ffi::add_ffi_modules;
use flint_utils::{app_err, Result};
use mlua::{Function, Lua, LuaSerdeExt, Table};
use std::{collections::HashMap, sync::Arc};

pub fn collect_env_vars(
    toml: &Arc<Config>,
    active_plugins: &Vec<Plugin>,
) -> HashMap<String, String> {
    let mut env_vars = HashMap::new();
    let lua = Lua::new();

    for plugin in active_plugins {
        let plugin_config = plugin.get_config_lua(&lua, toml);
        if let Ok(env) = plugin_config.get::<HashMap<String, String>>("env") {
            for (key, value) in env {
                env_vars.insert(key, value);
            }
        }
    }

    env_vars
}

pub fn generate<'a>(plugin: &Plugin, toml: &Arc<Config>) -> Result<HashMap<String, String>> {
    let lua = Lua::new();
    add_ffi_modules(&lua)?;

    let plugin_config = plugin.get_config_lua(&lua, toml);

    let generate: Function = {
        let contents = std::fs::read_to_string(plugin.path.join("generate.lua"))?;

        lua.load(contents)
            .exec()
            .map(|_| lua.globals().get("Generate").unwrap())
    }?;

    let validate: Function = {
        let contents = std::fs::read_to_string(plugin.path.join("validate.lua"))?;
        lua.load(contents)
            .exec()
            .map(|_| lua.globals().get("Validate").unwrap())
    }?;

    let validate_success = validate.call::<mlua::Value>(&plugin_config)?;

    let validate_success: bool = lua.from_value(validate_success)?;

    if !validate_success {
        return app_err!("Plugin configuration validation failed");
    }

    let generate_results = if plugin.kind == PluginKind::Ci {
        let active_plugins = crate::plugin::list_from_config(&toml);

        // Filter out CI plugins from active_plugins to avoid circular dependencies
        let active_plugins = active_plugins
            .into_iter()
            .filter(|p| p.kind != PluginKind::Ci)
            .collect::<Vec<_>>();

        let dependencies = collect_dependencies(&active_plugins)?;
        let env = collect_env_vars(&toml.clone(), &active_plugins);
        let env_table = lua.to_value(&env)?;

        let deps_table = lua.to_value(&dependencies)?;

        generate.call::<mlua::Value>((plugin_config, deps_table, env_table))
    } else {
        generate.call::<mlua::Value>(plugin_config)
    }?;

    let generate_results: HashMap<String, String> = lua.from_value(generate_results)?;

    Ok(generate_results)
}
