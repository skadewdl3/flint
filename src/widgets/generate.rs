use crate::util::{
    plugin::{list_plugins, run_plugin, Plugin, PluginDetails},
    toml::{read_toml_config, Config},
};

use super::{AppStatus, AppWidget};
use mlua::{Function, Lua, LuaSerdeExt};
use ratatui::Frame;

#[derive(Debug)]
pub struct GenerateWidget {
    plugins: Vec<Plugin>,
}

impl Default for GenerateWidget {
    fn default() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }
}

impl AppWidget for GenerateWidget {
    fn setup(&mut self) -> AppStatus {
        let toml = read_toml_config("./flint.toml").unwrap();
        let plugin_ids = toml.linters.keys().collect::<Vec<&String>>();
        let plugin_details = list_plugins();
        self.plugins = plugin_details
            .into_iter()
            .filter(|plugin| plugin_ids.contains(&&plugin.details.id))
            .collect();

        for plugin in &self.plugins {
            println!("Running plugin: {}", plugin.details.id);
            match run_plugin(&toml, plugin) {
                Ok(res) => {
                    println!("Res: {}", res)
                }
                Err(err) => {
                    return AppStatus::Error(err);
                }
            }
        }

        AppStatus::Ok
    }

    fn draw(&mut self, frame: &mut Frame) -> AppStatus {
        let items: Vec<ratatui::widgets::ListItem> = self
            .plugins
            .iter()
            .map(|plugin| ratatui::widgets::ListItem::new(plugin.details.id.clone()))
            .collect();
        let plugin_list = ratatui::widgets::List::new(items)
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Plugins"),
            )
            .highlight_style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
        // frame.render_widget(plugin_list, frame.area());
        AppStatus::Ok
    }
}
