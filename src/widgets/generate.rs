use crate::util::{
    plugin::{list_plugins, Plugin, PluginDetails},
    toml::{read_toml_config, Config},
};

use super::{AppStatus, AppWidget};
use mlua::{Function, Lua, LuaSerdeExt};
use ratatui::Frame;

#[derive(Debug)]
pub struct GenerateWidget {
    lua: Lua,
    plugins: Vec<Plugin>,
}

impl Default for GenerateWidget {
    fn default() -> Self {
        Self {
            lua: Lua::new(),
            plugins: Vec::new(),
        }
    }
}

impl GenerateWidget {
    pub fn run_plugin<'a>(&self, toml: &Config, plugin: &Plugin) -> AppStatus<'a> {
        let plugin_config = toml.linters.get(&plugin.details.id).unwrap();
        let plugin_config = self.lua.to_value(plugin_config).unwrap();
        let plugin_config = plugin_config.as_table().unwrap();

        let contents = match std::fs::read_to_string(&plugin.path) {
            Ok(contents) => contents,
            Err(_) => {
                return AppStatus::Error("Error reading plugin code");
            }
        };

        let (validate, _generate) = match self.lua.load(contents).exec() {
            Ok(_) => {
                let validate: Function = self.lua.globals().get("Validate").unwrap();
                let generate: Function = self.lua.globals().get("Generate").unwrap();
                (validate, generate)
            }
            Err(_) => {
                return AppStatus::Error("Error loading lua file");
            }
        };

        let _ = validate.call::<mlua::Value>(plugin_config).unwrap();

        AppStatus::Ok
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
            self.run_plugin(&toml, plugin);
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
