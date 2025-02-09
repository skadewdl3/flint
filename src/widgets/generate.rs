use crate::{util::lua::list_plugins, util::toml::read_toml_config};

use super::{AppStatus, AppWidget};
use mlua::Lua;
use ratatui::Frame;

#[derive(Debug)]
pub struct GenerateWidget {
    lua: Lua,
}

impl Default for GenerateWidget {
    fn default() -> Self {
        Self { lua: Lua::new() }
    }
}

impl AppWidget for GenerateWidget {
    fn setup(&mut self) -> AppStatus {
        let toml = read_toml_config("./flint.toml").unwrap();
        let x = list_plugins();
        println!("{:?}", toml);
        AppStatus::Ok
    }

    fn draw(&mut self, frame: &mut Frame) -> AppStatus {
        AppStatus::Ok
    }
}
