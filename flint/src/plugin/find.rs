use super::validate::validate_plugin_structure;
use super::{Plugin, PluginDetails, PluginKind};
use crate::util::toml::Config;
use flint_utils::{debug, error, get_flag, Result};
use mlua::{Function, Lua, LuaSerdeExt};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    sync::OnceLock,
};

pub static PLUGINS: OnceLock<BTreeSet<Plugin>> = OnceLock::new();
pub static PLUGIN_MAP: OnceLock<HashMap<String, BTreeSet<Plugin>>> = OnceLock::new();

pub fn map() -> &'static HashMap<String, BTreeSet<Plugin>> {
    PLUGIN_MAP.get_or_init(|| {
        let plugins = list().unwrap();
        let mut m = HashMap::new();
        for plugin in plugins {
            for extension in &plugin.details.extensions {
                m.entry(extension.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(plugin.clone());
            }
        }
        m
    })
}

pub fn list<'a>() -> Result<&'a BTreeSet<Plugin>> {
    let lua = Lua::new();
    flint_ffi::add_ffi_modules(&lua)?;

    if PLUGINS.get().is_some() {
        return Ok(PLUGINS.get().unwrap());
    }

    let plugins = ["lint", "test", "ci", "report"]
        .iter()
        .flat_map(|dir_name| {
            let plugins_dir = get_flag!(plugins_dir);
            let plugins_dir = plugins_dir.join(dir_name);
            if !plugins_dir.exists() {
                error!("{} directory does not exist", dir_name);
                return vec![];
            }

            let entries = match std::fs::read_dir(&plugins_dir) {
                Ok(entries) => entries,
                Err(e) => {
                    error!(
                        "Failed to read {} directory. Error message: {}",
                        dir_name, e
                    );
                    return vec![];
                }
            };

            entries
                .filter_map(|entry| {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(err) => {
                            error!("Error reading directory entry: {}", err);
                            return None;
                        }
                    };

                    let path = entry.path();
                    let contents = match std::fs::read_to_string(&path.join("details.lua")) {
                        Ok(contents) => contents,
                        Err(err) => {
                            error!("Error reading file {}: {}", path.display(), err);
                            return None;
                        }
                    };

                    match lua.load(contents).exec() {
                        Ok(_) => {
                            let details: Function = lua.globals().get("Details").unwrap();
                            let lua_val = details.call::<mlua::Value>(()).unwrap();
                            let details: PluginDetails = lua.from_value(lua_val).unwrap();

                            let plugin = Plugin {
                                details,
                                path,
                                kind: match *dir_name {
                                    "test" => PluginKind::Test,
                                    "lint" => PluginKind::Lint,
                                    "ci" => PluginKind::Ci,
                                    "report" => PluginKind::Report,
                                    _ => unreachable!(),
                                },
                            };

                            match validate_plugin_structure(&plugin) {
                                Ok(_) => Some(plugin),
                                Err(err) => {
                                    error!(
                                        "Plugin {} has invalid file structure.\nError message: {}",
                                        plugin.details.id, err
                                    );
                                    None
                                }
                            }
                        }

                        Err(err) => {
                            error!("Error loading lua file {}: {}", path.display(), err);
                            None
                        }
                    }
                })
                .collect::<Vec<_>>()
        });

    let x = PLUGINS.get_or_init(|| plugins.collect::<BTreeSet<Plugin>>());
    Ok(x)
}

pub fn list_from_config<'a>(config: &Config) -> Vec<&'a Plugin> {
    let linter_ids = config.rules.keys().collect::<HashSet<&String>>();
    let tester_ids = config.tests.keys().collect::<HashSet<&String>>();
    let ci_ids = config.ci.keys().collect::<HashSet<&String>>();
    let report_ids = config.report.keys().collect::<HashSet<&String>>();
    let plugins = list().unwrap();
    debug!("Loaded plugins: {:?}", plugins);

    plugins
        .iter()
        .filter(|plugin| {
            linter_ids.contains(&plugin.details.id)
                || tester_ids.contains(&plugin.details.id)
                || ci_ids.contains(&plugin.details.id)
                || report_ids.contains(&plugin.details.id)
        })
        .collect()
}
