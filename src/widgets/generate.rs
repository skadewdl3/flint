use crate::util::{
    lua::{list_plugins, PluginDetails},
    toml::read_toml_config,
};

use super::{AppStatus, AppWidget};
use mlua::Lua;
use ratatui::Frame;

#[derive(Debug)]
pub struct GenerateWidget {
    lua: Lua,
    plugins: Vec<PluginDetails>,
}

impl Default for GenerateWidget {
    fn default() -> Self {
        Self {
            lua: Lua::new(),
            plugins: Vec::new(),
        }
    }
}

impl AppWidget for GenerateWidget {
    fn setup(&mut self) -> AppStatus {
        let toml = read_toml_config("./flint.toml").unwrap();
        let plugin_details = list_plugins();
        self.plugins = plugin_details.into_iter().collect();

        AppStatus::Ok
    }

    fn draw(&mut self, frame: &mut Frame) -> AppStatus {
        let items: Vec<ratatui::widgets::ListItem> = self
            .plugins
            .iter()
            .map(|plugin| ratatui::widgets::ListItem::new(plugin.id.clone()))
            .collect();
        let plugin_list = ratatui::widgets::List::new(items)
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Plugins"),
            )
            .highlight_style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
        frame.render_widget(plugin_list, frame.area());
        AppStatus::Ok
    }
}
