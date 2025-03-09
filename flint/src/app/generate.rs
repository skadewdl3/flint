use super::{AppResult, AppWidget};
use crate::{
    util::{
        plugin::{self, Plugin},
        toml::Config,
    },
    widgets::logs::{add_log, LogKind, LogsWidget},
};
use clap::Parser;
use flint_macros::ui;
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use threadpool::ThreadPool;

pub struct GenerateWidget {
    plugins: Vec<Plugin>,
    thread_pool: ThreadPool,
    logs_widget: LogsWidget,
    args: GenerateWidgetArgs,
}

#[derive(Parser)]
pub struct GenerateWidgetArgs {
    /// Show help for the generate command
    #[clap(short, long)]
    help: bool,
}

impl GenerateWidget {
    pub fn new(args: GenerateWidgetArgs) -> Self {
        Self {
            plugins: Vec::new(),
            thread_pool: ThreadPool::new(16),
            logs_widget: LogsWidget::default(),
            args,
        }
    }
}

impl AppWidget for GenerateWidget {
    fn setup(&mut self) -> AppResult<()> {
        let toml = Arc::new(Config::load(PathBuf::from("./flint.toml")).unwrap());
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

            self.thread_pool.execute(move || {
                let result = plugin.generate(&toml_clone);
                match result {
                    Ok(res) => {
                        // TODO: Ask user if we want to overwrite files
                        for (file_name, contents) in res {
                            let flint_path = Path::new("./.flint");
                            fs::create_dir_all(&flint_path).unwrap();
                            std::fs::write(flint_path.join(file_name), contents).unwrap();
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

        Ok(())
    }
}

impl WidgetRef for GenerateWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        ui!((area, buf) => {
            { self.logs_widget }
        });
    }
}
