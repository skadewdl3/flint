use std::collections::BTreeSet;

use mlua::{Function, Lua, LuaSerdeExt};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PluginDetails {
    id: String,
    languages: Vec<String>,
    version: String,
    author: String,
}

pub fn list_plugins() -> BTreeSet<PluginDetails> {
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
                        println!("{:?}", details);
                    }
                    Err(err) => {
                        eprintln!("Error loading lua file {}: {}", file_path.display(), err);
                        continue;
                    }
                }

                // if let Some(file_name) = entry.file_name().to_str() {
                //     let details = PluginDetails {
                //         id: file_name.to_string(),
                //         langs: std::collections::BTreeSet::new(),
                //         version: 1,
                //         author: String::from("Unknown"),
                //     };
                //     plugins.insert(details);
                // }
            }
        }
    }
    plugins
}
