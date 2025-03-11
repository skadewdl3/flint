use directories::UserDirs;
use mlua::{Function, IntoLuaMulti, Lua, Table, Value};
use serde_json::to_string_pretty;

use crate::{app::AppResult, debug, error, get_flag, info, success, warn};

pub fn add_helper_globals(lua: &Lua) -> AppResult<()> {
    let log = lua.create_table()?;
    let json = lua.create_table()?;
    let toml = lua.create_table()?;
    let yaml = lua.create_table()?;
    let path = lua.create_table()?;

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

    let json_stringify = lua.create_function(|_, value: Value| match to_string_pretty(&value) {
        Ok(json) => Ok(json),
        Err(err) => Err(mlua::Error::external(err)),
    })?;
    let toml_stringify =
        lua.create_function(|_, value: Value| match toml::to_string_pretty(&value) {
            Ok(toml_str) => Ok(toml_str),
            Err(err) => Err(mlua::Error::external(err)),
        })?;

    let yaml_stringify =
        lua.create_function(|_, value: Value| match serde_yaml::to_string(&value) {
            Ok(yaml_str) => Ok(yaml_str),
            Err(err) => Err(mlua::Error::external(err)),
        })?;

    let json_parse = lua.create_function(|lua, json_str: String| {
        match serde_json::from_str::<serde_json::Value>(&json_str) {
            Ok(value) => serde_value_to_lua_value(lua, &value),
            Err(err) => Err(mlua::Error::external(err)),
        }
    })?;

    let toml_parse = lua.create_function(|lua, toml_str: String| {
        match toml::from_str::<toml::Value>(&toml_str) {
            Ok(value) => toml_value_to_lua_value(lua, &value),
            Err(err) => Err(mlua::Error::external(err)),
        }
    })?;

    let yaml_parse = lua.create_function(|lua, yaml_str: String| {
        match serde_yaml::from_str::<serde_yaml::Value>(&yaml_str) {
            Ok(value) => yaml_value_to_lua_value(lua, &value),
            Err(err) => Err(mlua::Error::external(err)),
        }
    })?;

    let cwd = lua.create_function(|lua, ()| {
        let cwd = get_flag!(current_dir);
        Ok(lua.create_string(cwd.to_string_lossy().as_ref())?)
    })?;

    let path_resolve = lua.create_function(|lua, paths: mlua::Variadic<String>| {
        use std::path::{Path, PathBuf};

        let cwd = get_flag!(current_dir);
        let mut result = PathBuf::new();
        let mut absolute = false;

        // Process each path segment similar to Node.js path.resolve
        for path in paths.iter() {
            let path_obj = Path::new(path);

            // If path is absolute, reset result and set absolute flag
            if path_obj.is_absolute() {
                result = PathBuf::from(path);
                absolute = true;
            } else if path.starts_with("~")
                && path.len() > 1
                && (path.len() == 1 || path.chars().nth(1) == Some('/'))
            {
                // Handle home directory with ~
                match UserDirs::new() {
                    Some(user_dirs) => {
                        let home = user_dirs.home_dir();
                        if path.len() > 1 {
                            result = home.join(&path[2..]);
                        } else {
                            result = home.to_path_buf();
                        }
                        absolute = true;
                    }
                    None => (),
                }
            } else {
                // For relative paths, append to result
                if !absolute {
                    // If this is the first path and it's relative, start from cwd
                    if result.as_os_str().is_empty() {
                        result = cwd.clone();
                    }
                }
                result = result.join(path);
            }
        }

        // If no paths provided, return cwd
        if result.as_os_str().is_empty() {
            result = cwd.clone();
        }

        // Normalize the path
        if let Ok(canonicalized) = result.canonicalize() {
            result = canonicalized;
        }

        Ok(lua.create_string(result.to_string_lossy().as_ref())?)
    })?;
    let path_join = lua.create_function(|lua, paths: mlua::Variadic<String>| {
        use std::path::Path;

        // If there are no path segments, return empty string
        if paths.len() == 0 {
            return Ok(lua.create_string("")?);
        }

        // Node.js path.join() just combines segments with the platform-specific separator
        // and normalizes the result, but it doesn't resolve to absolute paths
        let mut result = String::new();

        for (i, path) in paths.iter().enumerate() {
            // Skip empty segments (but preserve them at the beginning)
            if path.is_empty() && i > 0 {
                continue;
            }

            // Add separator between segments
            if i > 0 && !result.is_empty() && !result.ends_with(std::path::MAIN_SEPARATOR) {
                result.push(std::path::MAIN_SEPARATOR);
            }

            // Add the path segment
            result.push_str(path);
        }

        // Normalize the path (remove unnecessary separators/dots)
        let normalized = Path::new(&result).to_string_lossy();

        Ok(lua.create_string(normalized.as_ref())?)
    })?;

    log.set("info", create_log_fn("info"))?;
    log.set("error", create_log_fn("error"))?;
    log.set("warn", create_log_fn("warn"))?;
    log.set("success", create_log_fn("success"))?;
    log.set("debug", debug_print)?;

    json.set("stringify", json_stringify)?;
    json.set("parse", json_parse)?;

    toml.set("stringify", toml_stringify)?;
    toml.set("parse", toml_parse)?;

    yaml.set("stringify", yaml_stringify)?;
    yaml.set("parse", yaml_parse)?;

    path.set("cwd", cwd)?;
    path.set("resolve", path_resolve)?;
    path.set("join", path_join)?;

    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;

    // Register our module in package.loaded
    loaded.set("log", log)?;
    loaded.set("json", json)?;
    loaded.set("toml", toml)?;
    loaded.set("yaml", yaml)?;
    loaded.set("path", path)?;

    // Custom module loader to allow our modules to work
    lua.load(
        r#"
        -- Define our custom module resolver system
        local originalRequire = require

        function require(moduleName)
            -- Check if it's in package.loaded first
            if package.loaded[moduleName] then
                return package.loaded[moduleName]
            end

            -- If not found, fall back to the original require
            return originalRequire(moduleName)
        end
    "#,
    )
    .exec()?;

    // loaded.set("log", log_table.clone())?;
    // preload.set(
    //     "log",
    //     lua.create_function(move |lua, _: ()| Ok(log_table.clone()))?,
    // )?;

    Ok(())
}

// JSON value to Lua value converter
fn serde_value_to_lua_value<'lua>(
    lua: &'lua Lua,
    value: &serde_json::Value,
) -> Result<Value, mlua::Error> {
    match value {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(b) => Ok(Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i as i32))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Err(mlua::Error::external("Unsupported number format"))
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(lua.create_string(s)?)),
        serde_json::Value::Array(arr) => {
            let lua_table = lua.create_table()?;
            for (i, val) in arr.iter().enumerate() {
                lua_table.set(i + 1, serde_value_to_lua_value(lua, val)?)?;
            }
            Ok(Value::Table(lua_table))
        }
        serde_json::Value::Object(obj) => {
            let lua_table = lua.create_table()?;
            for (k, v) in obj.iter() {
                lua_table.set(k.clone(), serde_value_to_lua_value(lua, v)?)?;
            }
            Ok(Value::Table(lua_table))
        }
    }
}

// TOML value to Lua value converter
fn toml_value_to_lua_value<'lua>(
    lua: &'lua Lua,
    value: &toml::Value,
) -> Result<Value, mlua::Error> {
    match value {
        toml::Value::String(s) => Ok(Value::String(lua.create_string(s)?)),
        toml::Value::Integer(i) => Ok(Value::Integer(*i as i32)),
        toml::Value::Float(f) => Ok(Value::Number(*f)),
        toml::Value::Boolean(b) => Ok(Value::Boolean(*b)),
        toml::Value::Datetime(dt) => Ok(Value::String(lua.create_string(&dt.to_string())?)),
        toml::Value::Array(arr) => {
            let lua_table = lua.create_table()?;
            for (i, val) in arr.iter().enumerate() {
                lua_table.set(i + 1, toml_value_to_lua_value(lua, val)?)?;
            }
            Ok(Value::Table(lua_table))
        }
        toml::Value::Table(tbl) => {
            let lua_table = lua.create_table()?;
            for (k, v) in tbl.iter() {
                lua_table.set(k.clone(), toml_value_to_lua_value(lua, v)?)?;
            }
            Ok(Value::Table(lua_table))
        }
    }
}

// YAML value to Lua value converter
fn yaml_value_to_lua_value<'lua>(
    lua: &'lua Lua,
    value: &serde_yaml::Value,
) -> Result<Value, mlua::Error> {
    match value {
        serde_yaml::Value::Null => Ok(Value::Nil),
        serde_yaml::Value::Bool(b) => Ok(Value::Boolean(*b)),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i as i32))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Err(mlua::Error::external("Unsupported number format"))
            }
        }
        serde_yaml::Value::String(s) => Ok(Value::String(lua.create_string(s)?)),
        serde_yaml::Value::Sequence(seq) => {
            let lua_table = lua.create_table()?;
            for (i, val) in seq.iter().enumerate() {
                lua_table.set(i + 1, yaml_value_to_lua_value(lua, val)?)?;
            }
            Ok(Value::Table(lua_table))
        }
        serde_yaml::Value::Mapping(map) => {
            let lua_table = lua.create_table()?;
            for (k, v) in map.iter() {
                // Handle different key types for YAML mappings
                match k {
                    serde_yaml::Value::String(key_str) => {
                        lua_table.set(key_str.clone(), yaml_value_to_lua_value(lua, v)?)?;
                    }
                    serde_yaml::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            lua_table.set(i, yaml_value_to_lua_value(lua, v)?)?;
                        } else if let Some(f) = n.as_f64() {
                            // Convert float keys to strings
                            let key_str = lua.create_string(&f.to_string())?;
                            lua_table.set(key_str, yaml_value_to_lua_value(lua, v)?)?;
                        }
                    }
                    serde_yaml::Value::Bool(b) => {
                        // Convert boolean keys to strings
                        let key_str = lua.create_string(if *b { "true" } else { "false" })?;
                        lua_table.set(key_str, yaml_value_to_lua_value(lua, v)?)?;
                    }
                    _ => {
                        // For complex keys, serialize them to YAML and use that as the key
                        // This is a fallback for complex types
                        match serde_yaml::to_string(k) {
                            Ok(key_str) => {
                                // Trim the trailing newline that serde_yaml adds
                                let key_str = key_str.trim_end();
                                let lua_str = lua.create_string(key_str)?;
                                lua_table.set(lua_str, yaml_value_to_lua_value(lua, v)?)?;
                            }
                            Err(_) => {
                                // If serialization fails, use a placeholder
                                let key_str = lua.create_string("complex_key")?;
                                lua_table.set(key_str, yaml_value_to_lua_value(lua, v)?)?;
                            }
                        }
                    }
                }
            }
            Ok(Value::Table(lua_table))
        }
        serde_yaml::Value::Tagged(tagged) => {
            // Handle tagged values (just use the value)
            yaml_value_to_lua_value(lua, &tagged.value)
        }
    }
}
