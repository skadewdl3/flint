use clap::Parser;
use crossterm::event::{KeyCode, MouseEventKind};
use flint_macros::ui;
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;
use std::{cell::RefCell, fs, sync::Arc};
use threadpool::ThreadPool;

use crate::{
    plugin::{self, Plugin, PluginKind},
    util::{handle_key_events, handle_mouse_event, toml::Config},
    widgets::logs::{LogsState, LogsWidget},
};

use flint_utils::{error, get_flag, info, success, Result};

use super::AppWidget;

#[derive(Debug)]
pub struct TestWidget {
    logs: LogsWidget,
    thread_pool: Option<ThreadPool>,
    logs_state: RefCell<LogsState>,
    args: TestArgs,
}

#[derive(Parser, Debug, Clone)]
pub struct TestArgs {
    /// Show help for the test command
    #[clap(short, long)]
    help: bool,

    #[clap(short, long, default_value_t = true)]
    all: bool,

    #[clap(short, long)]
    lint: bool,

    #[clap(short, long)]
    test: bool,
}

impl TestWidget {
    pub fn new(args: TestArgs) -> Self {
        Self {
            thread_pool: None,
            logs: LogsWidget::default(),
            logs_state: RefCell::new(LogsState::default()),
            args,
        }
    }
}

impl AppWidget for TestWidget {
    fn setup(&mut self) -> Result<()> {
        let config_path = get_flag!(config_path);
        flint_utils::debug!("Config path: {:#?}", &config_path);
        let toml = Arc::new(Config::load(&config_path).unwrap());
        if let Some(ref env) = toml.flint.env {
            let cwd = get_flag!(current_dir);
            let env_path = cwd.join(env);
            if env_path.exists() {
                flint_utils::env::load_from_file(&env_path)?;
            }
        }
        let plugins = plugin::list_from_config(&toml);

        let run_plugins: Vec<Plugin> = plugins
            .clone()
            .iter()
            .filter(|plugin| plugin.kind != PluginKind::Report && plugin.kind != PluginKind::Ci)
            .filter(|plugin| {
                if !self.args.lint && !self.args.test && self.args.all {
                    true
                } else if self.args.lint {
                    plugin.kind == PluginKind::Lint
                } else if self.args.test {
                    plugin.kind == PluginKind::Test
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        let report_plugins: Arc<Vec<Plugin>> = Arc::new(
            plugins
                .iter()
                .filter(|plugin| plugin.kind == PluginKind::Report)
                .cloned()
                .collect(),
        );

        for plugin in run_plugins {
            let plugin = plugin.clone();
            let toml_clone = toml.clone();
            let report_plugins = Arc::clone(&report_plugins); // Share report plugins across threads
            let pool = self.thread_pool.as_ref().unwrap();

            pool.execute(move || {
                info!("Testing with: {}", plugin.details.id);
                let result = plugin.run(&toml_clone);

                if let Err(err) = result {
                    error!("{}", err);
                    return;
                }

                let command = result.unwrap();

                let cmd_output = std::process::Command::new(&command[0])
                    .args(&command[1..])
                    .current_dir(get_flag!(current_dir).as_path())
                    .output();

                info!("Running command: {:#?}", command);

                if let Err(e) = cmd_output {
                    error!("Failed to execute command '{}': {}", command[0], e);
                    return;
                }

                let output = cmd_output.unwrap();

                let eval_result = plugin.eval(output, &toml_clone);

                match eval_result {
                    Err(e) => error!("Failed to evaluate plugin: {}", e),
                    Ok(res) => {
                        for report_plugin in report_plugins.iter() {
                            info!("Running report plugin: {}", report_plugin.details.id);
                            match report_plugin.report(&toml_clone, &res, &plugin.details.id) {
                                Err(e) => {
                                    error!("Report plugin error: {}", e);
                                }
                                Ok(res) => {
                                    for (file_name, contents) in res {
                                        let flint_path = get_flag!(current_dir);
                                        let file_path = flint_path.join(&file_name);

                                        if let Some(parent) = file_path.parent() {
                                            if !parent.exists() {
                                                fs::create_dir_all(parent).unwrap_or_else(|e| {
                                                    error!(
                                                        "Failed to create directory for {}: {}",
                                                        file_name, e
                                                    );
                                                });
                                            }
                                        }

                                        match std::fs::write(flint_path.join(&file_name), contents)
                                        {
                                            Ok(_) => (),
                                            Err(e) => error!(
                                                "Failed to write report file {}: {}",
                                                file_name, e
                                            ),
                                        }

                                        success!(
                                            "Reported {} results to {} successfully",
                                            plugin.details.id,
                                            file_name
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }

        Ok(())
    }

    fn set_thread_pool(&mut self, thread_pool: &ThreadPool) {
        self.thread_pool = Some(thread_pool.clone());
    }

    fn handle_events(&mut self, event: crossterm::event::Event) -> Result<()> {
        let _ = handle_key_events(event.clone(), |_, key_code| match key_code {
            KeyCode::Up => {
                self.logs_state.borrow_mut().scroll_up(1);
                Ok(())
            }
            KeyCode::Down => {
                self.logs_state.borrow_mut().scroll_down(1);
                Ok(())
            }
            _ => Ok(()),
        });

        handle_mouse_event(event.clone(), |mouse_event| match mouse_event {
            MouseEventKind::ScrollUp => {
                self.logs_state.borrow_mut().scroll_up(1);
                Ok(())
            }
            MouseEventKind::ScrollDown => {
                self.logs_state.borrow_mut().scroll_down(1);
                Ok(())
            }
            _ => Ok(()),
        })
    }
}

impl WidgetRef for TestWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let mut logs_state = self.logs_state.borrow_mut();
        ui!((area, buf) => {
            Stateful(&mut logs_state) {
                { self.logs }
            }
        });
    }
}
