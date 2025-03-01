use std::{path::PathBuf, sync::Arc};

use flint_macros::ui;
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;
use threadpool::ThreadPool;
use throbber_widgets_tui::ThrobberState;

use crate::{
    util::{
        plugin::{self, Plugin, PluginKind},
        toml::Config,
    },
    widgets::logs::{add_log, LogKind, LogsWidget},
};

use super::{AppResult, AppWidget};

#[derive(Debug)]
pub struct TestWidget {
    logs: LogsWidget,
    thread_pool: ThreadPool,
}

impl Default for TestWidget {
    fn default() -> Self {
        Self {
            thread_pool: ThreadPool::new(16),
            logs: LogsWidget::default(),
        }
    }
}

impl AppWidget for TestWidget {
    fn setup(&mut self) -> AppResult<()> {
        let toml = Arc::new(Config::load(PathBuf::from("./flint.toml")).unwrap());
        let plugins = plugin::list_from_config();

        for plugin in plugins
            .into_iter()
            .filter(|plugin| plugin.kind == PluginKind::Lint)
        {
            let plugin = plugin.clone();
            let toml_clone = toml.clone();
            self.thread_pool.execute(move || {
                let result = plugin.run(&toml_clone);
                match result {
                    Ok(command) => {
                        match std::process::Command::new(&command[0])
                            .args(&command[1..])
                            .output()
                        {
                            Ok(_) => add_log(
                                LogKind::Success,
                                format!("Successfully executed command '{}'", command[0]),
                            ),
                            Err(e) => add_log(
                                LogKind::Error,
                                format!("Failed to execute command '{}': {}", command[0], e),
                            ),
                        };
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
impl WidgetRef for TestWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        ui!((area, buf) => {
            { self.logs }
        });
    }
}
