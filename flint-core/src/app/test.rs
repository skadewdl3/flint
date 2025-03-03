use std::{cell::RefCell, path::PathBuf, sync::Arc};

use crossterm::event::{KeyCode, MouseEventKind};
use flint_macros::ui;
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;
use threadpool::ThreadPool;

use crate::{
    util::{
        handle_key_events, handle_mouse_event,
        plugin::{self, PluginKind},
        toml::Config,
    },
    widgets::logs::{add_log, LogKind, LogsState, LogsWidget},
};

use super::{AppResult, AppWidget};

#[derive(Debug)]
pub struct TestWidget {
    logs: LogsWidget,
    thread_pool: ThreadPool,
    logs_state: RefCell<LogsState>,
}

impl Default for TestWidget {
    fn default() -> Self {
        Self {
            thread_pool: ThreadPool::new(16),
            logs: LogsWidget::default(),
            logs_state: RefCell::new(LogsState::default()),
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

    fn handle_events(&mut self, event: crossterm::event::Event) -> AppResult<()> {
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
                add_log(LogKind::Info, "Scroll up".to_string());
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
