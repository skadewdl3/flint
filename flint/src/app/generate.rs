use super::{AppResult, AppWidget};
use crate::{
    get_flag, success,
    util::{
        plugin::{self, Plugin},
        toml::Config,
    },
    widgets::logs::LogsWidget,
};
use clap::Parser;
use flint_macros::ui;
use log::error;
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;
use std::{fs, sync::Arc};
use threadpool::ThreadPool;

#[allow(unused)]
pub struct GenerateWidget {
    plugins: Vec<Plugin>,
    thread_pool: Option<ThreadPool>,
    logs_widget: LogsWidget,
    args: GenerateWidgetArgs,
}

#[derive(Parser, Clone)]
pub struct GenerateWidgetArgs {
    /// Show help for the generate command
    #[clap(short, long)]
    help: bool,
}

impl GenerateWidget {
    pub fn new(args: GenerateWidgetArgs) -> Self {
        Self {
            plugins: Vec::new(),
            thread_pool: None,
            logs_widget: LogsWidget::default(),
            args,
        }
    }
}

impl AppWidget for GenerateWidget {
    fn setup(&mut self) -> AppResult<()> {
        let config_path = get_flag!(config_path);
        let toml = Arc::new(Config::load(config_path).unwrap());
        let mut plugin_ids = Vec::new();
        plugin_ids.extend(toml.rules.keys());
        plugin_ids.extend(toml.tests.keys());
        plugin_ids.extend(toml.ci.keys());

        self.plugins = plugin::list()
            .unwrap()
            .into_iter()
            .filter(|plugin| plugin_ids.contains(&&plugin.details.id))
            .cloned()
            .collect();

        for plugin in &self.plugins {
            let plugin = plugin.clone();
            let toml_clone = toml.clone();
            let pool = self.thread_pool.as_ref().unwrap();

            pool.execute(move || {
                let result = plugin.generate(&toml_clone);
                match result {
                    Ok(res) => {
                        // TODO: Ask user if we want to overwrite files
                        let flint_path = get_flag!(current_dir).join(".flint");
                        for (file_name, contents) in res {
                            fs::create_dir_all(&flint_path).unwrap();
                            std::fs::write(flint_path.join(file_name), contents).unwrap();
                        }
                        success!("Generated {} config successfully", plugin.details.id)
                    }
                    Err(err) => {
                        error!("Error occurred: {}", err);
                    }
                }
            });
        }

        Ok(())
    }

    fn set_thread_pool(&mut self, thread_pool: &ThreadPool) {
        self.thread_pool = Some(thread_pool.clone())
    }
}

impl WidgetRef for GenerateWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        ui!((area, buf) => {
            { self.logs_widget }
        });
    }
}
