use std::{
    collections::{BTreeSet, HashMap},
    sync::{mpsc, Arc, Mutex},
};

use mlua::{Function, Lua, Value};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::Text,
    Frame,
};
use threadpool::ThreadPool;

use super::{AppStatus, AppWidget};
use crate::util::{
    get_plugin_map,
    plugin::{run_plugin, Plugin},
    toml::read_toml_config,
};

pub struct GenerateWidget {
    plugins: Vec<Plugin>,
    thread_pool: ThreadPool,
    logs: Arc<Mutex<Vec<String>>>,
}

impl Default for GenerateWidget {
    fn default() -> Self {
        Self {
            plugins: Vec::new(),
            thread_pool: ThreadPool::new(16),
            logs: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl AppWidget for GenerateWidget {
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
            let plugin = plugin.clone();
            let toml_clone = toml.clone();
            let logs_clone = self.logs.clone();

            self.thread_pool.execute(move || {
                let result = run_plugin(&plugin, &toml_clone, logs_clone.clone());
                if let Ok(mut logs) = logs_clone.lock() {
                    match result {
                        Ok(_) => logs.push(format!(
                            "Generated {} config successfully",
                            plugin.details.id
                        )),
                        Err(err) => {
                            logs.push(format!("Error in {}: {}", plugin.details.id, err));
                        }
                    }
                }
            });
        }

        AppStatus::Ok
    }

    fn draw(&mut self, frame: &mut Frame) -> AppStatus {
        if let Ok(logs) = self.logs.lock() {
            let length = logs.len();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(Constraint::from_lengths(vec![1; length]))
                .split(frame.area());

            for i in 0..logs.len() {
                frame.render_widget(Text::raw(logs[i].clone()), layout[i]);
            }
        }

        AppStatus::Ok
    }
}
