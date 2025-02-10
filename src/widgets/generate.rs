use std::{collections::BTreeSet, sync::Arc};

use ratatui::Frame;

use super::{AppStatus, AppWidget};
use crate::util::{
    get_plugin_map,
    plugin::{run_plugin, Plugin},
    thread_manager::ThreadManager,
    toml::read_toml_config,
};

pub struct GenerateWidget {
    plugins: Vec<Plugin>,
    thread_manager: Option<ThreadManager>,
}

impl Default for GenerateWidget {
    fn default() -> Self {
        Self {
            plugins: Vec::new(),
            thread_manager: None,
        }
    }
}

impl AppWidget for GenerateWidget {
    fn register_thread_manager(&mut self, thread_manager: ThreadManager) {
        self.thread_manager = Some(thread_manager)
    }

    fn setup(&mut self) -> AppStatus {
        let toml = Arc::new(read_toml_config("./flint.toml").unwrap());
        let plugin_ids = toml.linters.keys().collect::<Vec<&String>>();

        self.plugins = get_plugin_map()
            .values()
            .flat_map(|plugin_set| plugin_set.iter())
            .collect::<BTreeSet<&Plugin>>()
            .into_iter()
            .filter(|plugin| plugin_ids.contains(&&plugin.details.id))
            .cloned()
            .collect();

        for plugin in &self.plugins {
            println!("Running plugin: {}", plugin.details.id);
            let plugin_clone = plugin.clone();
            let toml_clone = toml.clone();
            let handle = std::thread::spawn(move || run_plugin(toml_clone, &plugin_clone));
            if let Some(thread_manager) = &self.thread_manager {
                thread_manager.add_thread(handle);
            }
        }

        AppStatus::Ok
    }

    fn draw(&mut self, frame: &mut Frame) -> AppStatus {
        AppStatus::Ok
    }
}
