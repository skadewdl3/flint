use crate::{
    app::AppResult,
    app_err,
    plugin::{helpers::add_helper_globals, Plugin},
};
use mlua::{Error, Function, Lua, LuaSerdeExt, Table, Value};
use serde::{Deserialize, Serialize};
use std::process::Output;

#[derive(Serialize, Deserialize, Debug)]
pub struct TestCaseOutput {
    file_name: String,
    line_no: Option<u32>, // Default values if not available
    column_no: Option<u32>,
    success: bool, // Converted from assertion.status == "passed"
    error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LintPluginEvalOutput {
    total_errors: u32,
    lint_results: Vec<TestCaseOutput>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TestPluginEvalOutput {
    tests_passed: u32,
    total_tests: u32,
    passing_percentage: f32,
    test_results: Vec<TestCaseOutput>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PluginEvalOutput {
    Lint(LintPluginEvalOutput),
    Test(TestPluginEvalOutput),
}

pub fn eval(plugin: &Plugin, output: Output) -> AppResult<PluginEvalOutput> {
    let lua = Lua::new();
    add_helper_globals(&lua)?;

    let eval: Result<Function, Error> = {
        let contents = std::fs::read_to_string(plugin.path.join("run.lua"))?;

        lua.load(contents)
            .exec()
            .map(|_| lua.globals().get("Eval").unwrap())
    };

    let evaluation_state = lua.create_table().unwrap();
    evaluation_state.set("stdout", String::from_utf8_lossy(&output.stdout))?;
    evaluation_state.set("stderr", String::from_utf8_lossy(&output.stderr))?;
    evaluation_state.set("status", output.status.code())?;

    evaluation_state.set("success", output.status.success())?;

    let eval_output = eval
        .expect("error reading run.lua")
        .call::<mlua::Value>(evaluation_state)?;

    let eval_output_table = match &eval_output {
        Value::Table(table) => table,
        _ => return Err(app_err!("Eval function should return a valid lua table")),
    };

    if eval_output_table.contains_key("test_results")? {
        let test_output: TestPluginEvalOutput = lua
            .from_value(eval_output.clone())?
        Ok(PluginEvalOutput::Test(test_output))
    } else if eval_output_table.contains_key("lint_results")? {
        let lint_output: LintPluginEvalOutput = lua
            .from_value(eval_output.clone())?;
        Ok(PluginEvalOutput::Lint(lint_output))
    } else {
        Err(app_err!("Unknown plugin output format"))
    }
}
