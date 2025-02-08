use super::AppWidget;
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
    fn setup(&mut self) {
        self.lua.load("print('Hello, World!')").exec().unwrap();
    }

    fn draw(&mut self, frame: &mut Frame) {}
}
