use std::{collections::BTreeSet, path::PathBuf};

use mlua::{Function, Lua, LuaSerdeExt};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct PluginDetails {
    pub id: String,
    pub languages: Vec<String>,
    pub version: String,
    pub author: String,
}

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Plugin {
    pub details: PluginDetails,
    pub path: PathBuf,
}

pub fn list_plugins() -> BTreeSet<Plugin> {
    let lua = Lua::new();

    let mut plugins = BTreeSet::new();
    if let Ok(entries) = std::fs::read_dir("src/plugins") {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_path = entry.path();
                let contents = match std::fs::read_to_string(&file_path) {
                    Ok(contents) => contents,
                    Err(err) => {
                        eprintln!("Error reading file {}: {}", file_path.display(), err);
                        continue;
                    }
                };

                match lua.load(contents).exec() {
                    Ok(_) => {
                        let details: Function = lua.globals().get("Details").unwrap();
                        let lua_val = details.call::<mlua::Value>(()).unwrap();
                        let details: PluginDetails = lua.from_value(lua_val).unwrap();
                        plugins.insert(Plugin {
                            details,
                            path: file_path,
                        });
                    }
                    Err(err) => {
                        eprintln!("Error loading lua file {}: {}", file_path.display(), err);
                        continue;
                    }
                }
            }
        }
    }
    plugins
}
