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
        let lua = &self.lua;
        AppStatus::Ok
    }

    fn draw(&mut self, frame: &mut Frame) -> AppStatus {
        AppStatus::Ok
    }
}
