use ratatui::Frame;

use super::{AppStatus, AppWidget};
use crate::util::{
    plugin::{list_plugins, run_plugin, Plugin},
    toml::read_toml_config,
};

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
        AppStatus::Ok
    }
}
