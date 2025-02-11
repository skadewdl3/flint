use std::{
    collections::{BTreeSet, HashMap},
    sync::{mpsc, Arc, Mutex, RwLock},
};

use color_eyre::owo_colors::OwoColorize;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::Text,
    Frame,
};
use threadpool::ThreadPool;

use super::{
    logs::{add_log, LogKind, LogsWidget},
    AppStatus, AppWidget,
};
use crate::util::{
    get_plugin_map,
    plugin::{run_plugin, Plugin},
    toml::read_toml_config,
};

pub struct GenerateWidget {
    plugins: Vec<Plugin>,
    thread_pool: ThreadPool,
    logs_widget: LogsWidget,
}

impl Default for GenerateWidget {
    fn default() -> Self {
        Self {
            plugins: Vec::new(),
            thread_pool: ThreadPool::new(16),
            logs_widget: LogsWidget::default(),
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

            self.thread_pool.execute(move || {
                let result = run_plugin(&plugin, &toml_clone);
                match result {
                    Ok(res) => {
                        for (file_name, contents) in res {
                            std::fs::write(file_name, contents).unwrap();
                        }
                        add_log(LogKind::Success, "".to_string());
                    }
                    Err(err) => {
                        add_log(LogKind::Error, format!("{}", err).to_string());
                    }
                }
            });
        }

        AppStatus::Ok
    }

    fn draw(&mut self, frame: &mut Frame) -> AppStatus {
        self.logs_widget.draw(frame);
        AppStatus::Ok
    }
}
