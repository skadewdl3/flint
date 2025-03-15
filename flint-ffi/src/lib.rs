use flint_utils::Result;
use mlua::{Lua, Table};

mod ai;
mod coroutine;
mod eval;
mod js;
mod json;
mod log;
mod md;
mod path;
mod sql;
mod toml;
mod yaml;

pub fn add_ffi_modules(lua: &Lua) -> Result<()> {
    let log = log::log_helpers(lua)?;
    let json = json::json_helpers(lua)?;
    let toml = toml::toml_helpers(lua)?;
    let yaml = yaml::yaml_helpers(lua)?;
    let path = path::path_helpers(lua)?;
    let js = js::js_helpers(lua)?;
    let eval = eval::eval_helpers(lua)?;
    let sql = sql::sql_helpers(lua)?;
    let coroutine = coroutine::coroutine_helpers(lua)?;
    let md = md::md_helpers(lua)?;
    let ai = ai::ai_helpers(lua)?;

    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;

    // Register our module in package.loaded
    loaded.set("log", log)?;
    loaded.set("json", json)?;
    loaded.set("toml", toml)?;
    loaded.set("yaml", yaml)?;
    loaded.set("path", path)?;
    loaded.set("js", js)?;
    loaded.set("eval", eval)?;
    loaded.set("sql", sql)?;
    loaded.set("async", coroutine)?;
    loaded.set("md", md)?;
    loaded.set("ai", ai)?;

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
