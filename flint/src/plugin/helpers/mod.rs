use mlua::{Lua, Table};

pub mod js;
pub mod json;
pub mod log;
pub mod path;
pub mod toml;
pub mod yaml;

use crate::app::AppResult;

pub fn add_helper_globals(lua: &Lua) -> AppResult<()> {
    let log = log::log_helpers(lua)?;
    let json = json::json_helpers(lua)?;
    let toml = toml::toml_helpers(lua)?;
    let yaml = yaml::yaml_helpers(lua)?;
    let path = path::path_helpers(lua)?;
    let js = js::js_helpers(lua)?;

    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;

    // Register our module in package.loaded
    loaded.set("log", log)?;
    loaded.set("json", json)?;
    loaded.set("toml", toml)?;
    loaded.set("yaml", yaml)?;
    loaded.set("path", path)?;
    loaded.set("js", js)?;

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

    Ok(())
}
