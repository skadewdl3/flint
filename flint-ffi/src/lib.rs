use std::sync::Arc;

use flint_utils::{Result, debug};
use mlua::{Lua, Table};

mod ai;
mod cmd;
mod coroutine;
mod csv;
mod env;
mod eval;
mod js;
mod json;
mod log;
mod md;
mod path;
mod sql;
mod toml;
mod yaml;

use ignore::WalkBuilder;
use std::fs;
use std::path::PathBuf;

/// Walks all files in the given directory (respecting .gitignore) and returns their contents in Markdown code blocks.
fn generate_markdown_from_files(dir: &str) -> String {
    let mut output = String::new();
    let base_path = PathBuf::from(dir);

    for result in WalkBuilder::new(dir).hidden(false).git_ignore(true).build() {
        if let Ok(entry) = result {
            let path = entry.path();
            if path.is_file() {
                if let Ok(relative_path) = path.strip_prefix(&base_path) {
                    let relative_path_str = relative_path.to_string_lossy();
                    output.push_str(&format!("\n**File:** `{}`\n", relative_path_str));
                }

                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    output.push_str(&format!("```{}\n", ext));
                } else {
                    output.push_str("```\n");
                }

                if let Ok(contents) = fs::read_to_string(path) {
                    output.push_str(&contents);
                } else {
                    output.push_str("[Could not read file]");
                }

                output.push_str("\n```\n");
            }
        }
    }

    output
}

pub fn add_ffi_modules(lua: &Lua) -> Result<()> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let rt = Arc::new(rt);

    let log = log::log_helpers(lua)?;
    let json = json::json_helpers(lua)?;
    let toml = toml::toml_helpers(lua)?;
    let yaml = yaml::yaml_helpers(lua)?;
    let path = path::path_helpers(lua)?;
    let js = js::js_helpers(lua)?;
    let eval = eval::eval_helpers(lua)?;
    let sql = sql::sql_helpers(lua)?;
    let coroutine = coroutine::coroutine_helpers(lua, rt.clone())?;
    let md = md::md_helpers(lua)?;
    let ai = ai::ai_helpers(lua)?;
    let env = env::env_helpers(lua)?;
    let csv = csv::csv_helpers(lua)?;
    let cmd = cmd::command_helpers(lua)?;

    let package: Table = lua.globals().get("package")?;
    let loaded: Table = package.get("loaded")?;
    let fs = lua.create_table()?;
    fs.set(
        "get_ai_input",
        lua.create_function(|_, dir: String| {
            debug!("{}", dir);
            let res = generate_markdown_from_files(&dir);
            Ok(res)
        })?,
    )?;

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
    loaded.set("env", env)?;
    loaded.set("csv", csv)?;
    loaded.set("cmd", cmd)?;
    loaded.set("fs", fs)?;

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
