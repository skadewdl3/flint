use super::toml::Config;
use crate::app::{AppError, AppResult};

pub mod find;
pub mod helpers;
pub use find::*;
pub mod download;

use helpers::add_helper_globals;

use mlua::{Error, Function, Lua, LuaSerdeExt, Table};
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

#[derive(Serialize, Deserialize, Debug)]
pub struct TestCaseOutput {
    file_name: String,
    line_no: Option<u32>, // Default values if not available
    column_no: Option<u32>,
    success: bool, // Converted from assertion.status == "passed"
    error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PluginEvalOutput {
    tests_passed: u32,
    total_tests: u32,
    passing_percentage: f32,
    test_results: Vec<TestCaseOutput>,
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

    pub fn generate<'a>(&self, toml: &Arc<Config>) -> AppResult<HashMap<String, String>> {
        let lua = Lua::new();
        add_helper_globals(&lua);

        let plugin_config = &self.get_config_lua(&lua, toml);

        let generate: Result<Function, Error> = {
            let contents = std::fs::read_to_string(&self.path.join("generate.lua"))?;

            lua.load(contents)
                .exec()
                .map(|_| lua.globals().get("Generate").unwrap())
        };

        let validate: Result<Function, Error> = {
            let contents = std::fs::read_to_string(&self.path.join("validate.lua"))?;
            lua.load(contents)
                .exec()
                .map(|_| lua.globals().get("Validate").unwrap())
        };

        let validate_success = validate
            .expect("error reading validate.lua")
            .call::<mlua::Value>(plugin_config)
            .expect("error running validate function");

        let validate_success: bool = lua
            .from_value(validate_success)
            .expect("unable to convert validation result to boolean");

        if !validate_success {
            return Err(AppError::Err(
                "Plugin configuration validation failed".into(),
            ));
        }

        let generate_results = generate
            .expect("Error reading generate.lua")
            .call::<mlua::Value>(plugin_config)
            .expect("error running generate function");
        let generate_results: HashMap<String, String> = lua
            .from_value(generate_results)
            .expect("unable to convert generation result to String");

        Ok(generate_results)
    }

    pub fn run<'a>(&self, toml: &Arc<Config>) -> AppResult<Vec<String>> {
        let lua = Lua::new();
        add_helper_globals(&lua);
        let plugin_config = &self.get_config_lua(&lua, toml);

        let run: Result<Function, Error> = {
            let contents = std::fs::read_to_string(&self.path.join("run.lua"))
                .expect("Error reading plugin code");

            lua.load(contents)
                .exec()
                .map(|_| lua.globals().get("Run").unwrap())
        };

        let run_success = run
            .expect("error reading run.lua")
            .call::<mlua::Value>(plugin_config)
            .expect("error running run function");

        let run_command: Vec<String> = lua
            .from_value(run_success)
            .expect("unable to parse run command");

        Ok(run_command)
    }

    pub fn eval(&self, output: Output) -> AppResult<PluginEvalOutput> {
        let lua = Lua::new();
        add_helper_globals(&lua);

        let eval: Result<Function, Error> = {
            let contents = std::fs::read_to_string(&self.path.join("run.lua"))
                .expect("Error reading plugin code");

            lua.load(contents)
                .exec()
                .map(|_| lua.globals().get("Eval").unwrap())
        };

        let evaluation_state = lua.create_table().unwrap();
        evaluation_state
            .set("stdout", String::from_utf8_lossy(&output.stdout))
            .unwrap();
        evaluation_state
            .set("stderr", String::from_utf8_lossy(&output.stderr))
            .unwrap();
        evaluation_state
            .set("status", output.status.code())
            .unwrap();

        evaluation_state
            .set("success", output.status.success())
            .unwrap();

        let eval_output = eval
            .expect("error reading run.lua")
            .call::<mlua::Value>(evaluation_state)
            .expect("error running eval function");

        let eval_output: PluginEvalOutput = lua
            .from_value(eval_output)
            .expect("unable to parse eval success");

        Ok(eval_output)
    }

    pub fn report(
        &self,
        toml: &Arc<Config>,
        output: &PluginEvalOutput,
    ) -> AppResult<HashMap<String, String>> {
        if self.kind != PluginKind::Report {
            return Err(AppError::Err(format!(
                "{} is not a reporting plugin.",
                self.details.id
            )));
        }

        let lua = Lua::new();
        add_helper_globals(&lua);

        let plugin_config = &self.get_config_lua(&lua, toml);

        let report: Result<Function, Error> = {
            let contents = std::fs::read_to_string(&self.path.join("run.lua"))
                .expect("Error reading plugin code");

            lua.load(contents)
                .exec()
                .map(|_| lua.globals().get("Run").unwrap())
        };

        let report_state = lua.create_table().unwrap();
        report_state.set("config", plugin_config).unwrap();
        let output_lua = lua.to_value(&output).unwrap();
        report_state.set("output", output_lua).unwrap();

        let report_results = report
            .expect("error reading run.lua")
            .call::<mlua::Value>(report_state)
            .expect("error running report function");

        let report_results: HashMap<String, String> = lua
            .from_value(report_results)
            .expect("unable to convert generation result to String");

        Ok(report_results)
    }

    pub fn list_from_config(config: &Config) -> Vec<&Plugin> {
        let linter_ids = config.rules.keys().collect::<HashSet<&String>>();
        let plugins = find::list().unwrap();

        plugins
            .iter()
            .filter(|plugin| linter_ids.contains(&plugin.details.id))
            .collect()
    }
}
