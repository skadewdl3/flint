use super::toml::Config;
use crate::app::AppResult;

pub mod find;
pub mod helpers;
use eval::PluginEvalOutput;
pub use find::*;
pub mod download;
pub mod eval;
pub mod generate;
pub mod report;
pub mod run;
pub mod validate;

use mlua::{Lua, LuaSerdeExt, Table};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    process::Output,
    sync::Arc,
};

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct PluginDetails {
    pub id: String,
    pub extensions: Vec<String>,
    pub version: String,
    pub author: String,
}

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Plugin {
    pub details: PluginDetails,
    pub path: PathBuf,
    pub kind: PluginKind,
}

#[derive(Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum PluginKind {
    Lint,
    Test,
    Ci,
    Report,
}

impl PluginKind {
    pub fn to_string(&self) -> String {
        match self {
            PluginKind::Lint => "lint".to_string(),
            PluginKind::Test => "test".to_string(),
            PluginKind::Ci => "ci".to_string(),
            PluginKind::Report => "report".to_string(),
        }
    }
}

impl Plugin {
    pub fn get_config_lua(&self, lua: &Lua, toml: &Arc<Config>) -> Table {
        let common_config = lua
            .to_value(&toml.common)
            .expect("unable to convert common config to lua value");

        let plugin_config = match self.kind {
            PluginKind::Lint => toml.rules.get(&self.details.id),
            PluginKind::Test => toml.tests.get(&self.details.id),
            PluginKind::Ci => toml.ci.get(&self.details.id),
            PluginKind::Report => toml.report.get(&self.details.id),
        }
        .expect(format!("unable to find config for plugin - {}", self.details.id).as_str());

        let plugin_config = lua
            .to_value(plugin_config)
            .expect("unable to convert plugin config to lua value");
        let plugin_config = plugin_config
            .as_table()
            .expect("unable to convert plugin config lua value to table");

        plugin_config
            .set("common", common_config)
            .expect("unable to set common table to config table");

        if self.kind == PluginKind::Lint {
            if let Some(temp) = toml.config.get(&self.details.id) {
                let extra_config = lua.to_value(temp).expect(
                    format!(
                        "unable to convert config.{}.extra to lua value",
                        self.details.id
                    )
                    .as_str(),
                );
                plugin_config.set("config", extra_config).expect(&format!(
                    "unable to set extra config for plugin {}",
                    self.details.id
                ))
            }
        }

        plugin_config.clone()
    }

    pub fn generate(&self, toml: &Arc<Config>) -> AppResult<HashMap<String, String>> {
        generate::generate(&self, toml)
    }

    pub fn run<'a>(&self, toml: &Arc<Config>) -> AppResult<Vec<String>> {
        run::run(&self, toml)
    }

    pub fn eval(&self, output: Output) -> AppResult<PluginEvalOutput> {
        eval::eval(&self, output)
    }

    pub fn report(
        &self,
        toml: &Arc<Config>,
        output: &PluginEvalOutput,
    ) -> AppResult<HashMap<String, String>> {
        report::report(&self, toml, output)
    }
}

pub fn list_from_config(config: &Arc<Config>) -> Vec<Plugin> {
    let mut plugin_ids = Vec::new();
    plugin_ids.extend(config.rules.keys());
    plugin_ids.extend(config.tests.keys());

    let plugins = find::list().unwrap();

    plugins
        .iter()
        .filter(|plugin| plugin_ids.contains(&&plugin.details.id))
        .cloned()
        .collect()
}
