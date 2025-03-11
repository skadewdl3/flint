use crate::{
    app::AppResult,
    plugin::{helpers::add_helper_globals, Plugin},
};
use mlua::{Error, Function, Lua, LuaSerdeExt};
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
pub struct PluginEvalOutput {
    tests_passed: u32,
    total_tests: u32,
    passing_percentage: f32,
    test_results: Vec<TestCaseOutput>,
}

pub fn eval(plugin: &Plugin, output: Output) -> AppResult<PluginEvalOutput> {
    let lua = Lua::new();
    add_helper_globals(&lua)?;

    let eval: Result<Function, Error> = {
        let contents = std::fs::read_to_string(plugin.path.join("run.lua"))
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
