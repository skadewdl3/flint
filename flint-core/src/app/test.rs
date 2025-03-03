use std::{path::PathBuf, sync::Arc};

use flint_macros::ui;
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;
use threadpool::ThreadPool;

use crate::{
    util::{
        plugin::{self, PluginKind},
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
            .filter(|plugin| plugin.kind == PluginKind::Test)
        {
            let plugin = plugin.clone();
            let toml_clone = toml.clone();
            self.thread_pool.execute(move || {
                let result = plugin.run(&toml_clone);

                if let Err(err) = result {
                    add_log(LogKind::Error, err.to_string());
                    return;
                }

                let command = result.unwrap();

                let cmd_output = std::process::Command::new(&command[0])
                    .args(&command[1..])
                    .output();

                if let Err(e) = cmd_output {
                    add_log(
                        LogKind::Error,
                        format!("Failed to execute command '{}': {}", command[0], e),
                    );
                    return;
                }

                let output = cmd_output.unwrap();

                let eval_result = plugin.eval(output);

                if let Err(eval) = eval_result {
                    add_log(
                        LogKind::Error,
                        format!("Failed to evaluate plugin: {}", eval),
                    );
                    return;
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
