use super::{
    logs::{add_log, LogKind, LogsWidget},
    AppStatus, AppWidget,
};
use crate::util::{
    get_plugin_map,
    plugin::{run_plugin, Plugin},
    toml::read_toml_config,
};
use flint_macros::ui;
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;
use std::{collections::BTreeSet, sync::Arc};
use threadpool::ThreadPool;

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
                        add_log(
                            LogKind::Success,
                            format!("Generated {} config successfully", plugin.details.id),
                        );
                    }
                    Err(err) => {
                        add_log(LogKind::Error, err.to_string());
                    }
                }
            });
        }

        AppStatus::Ok
    }
}

impl WidgetRef for GenerateWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        ui!((area, buf) => {
            { self.logs_widget }
        });
    }
}
