use std::cell::RefCell;
use std::sync::mpsc::Sender;
use std::time::Duration;

use crate::plugin::download::download_plugins_from_config;
use crate::util::toml::Config;
use crate::util::{handle_key_events, handle_mouse_event};
use crate::widgets::logs::{LogsState, LogsWidget};
use crate::{error, get_flag, success, warn};
use clap::Parser;
use crossterm::event::{KeyCode, MouseEventKind};
use threadpool::ThreadPool;

use super::{AppResult, AppWidget};
use flint_macros::ui;
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;

#[derive(Debug)]
#[allow(unused)]
pub struct InstallWidget {
    logs: LogsWidget,
    args: InstallArgs,
    pool: Option<ThreadPool>,
    exit_sender: Option<Sender<()>>,
    logs_state: RefCell<LogsState>,
}

#[derive(Parser, Debug, Clone)]
pub struct InstallArgs {
    /// List of plugins to install
    #[clap(short, long, value_parser, num_args = 1, value_delimiter = ' ')]
    plugins: Vec<String>,

    /// Install all linting plugins
    #[clap(short, long)]
    lint: bool,

    /// Install all testing plugins
    #[clap(short, long)]
    test: bool,

    /// Install all plugins (this is the default behaviour)
    #[clap(long, default_value_t = true)]
    all: bool,

    /// Show help for the install command
    #[clap(short, long)]
    help: bool,

    /// Show logs
    #[clap(long)]
    logs: bool,
}

impl InstallWidget {
    pub fn new(args: InstallArgs) -> Self {
        Self {
            logs: LogsWidget::default(),
            pool: None,
            exit_sender: None,
            logs_state: RefCell::new(LogsState::default()),
            args,
        }
    }
}

impl AppWidget for InstallWidget {
    fn setup(&mut self) -> AppResult<()> {
        if *get_flag!(no_install) {
            warn!("Skipping installation of plugins due to --no-install flag");
            return Ok(());
        };

        let toml = Config::load(get_flag!(config_path)).unwrap();
        let toml_clone = toml.clone();
        let pool = self.pool.as_ref().unwrap();
        pool.execute(move || {
            std::thread::sleep(Duration::from_secs(10));
            match download_plugins_from_config(&toml_clone) {
                Ok(_) => success!("Plugins downloaded successfully"),
                Err(e) => error!("Error downloading plugins: {}", e),
            }
        });
        Ok(())
    }

    fn set_exit_sender(&mut self, exit_sender: Sender<()>) {
        self.exit_sender = Some(exit_sender);
    }

    fn set_thread_pool(&mut self, thread_pool: &ThreadPool) {
        self.pool = Some(thread_pool.clone())
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

impl WidgetRef for InstallWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        // Check if there are no active threads in the threadpool
        // if self.pool.active_count() == 0 && self.pool.queued_count() == 0 {
        //     // self.exit_sender.as_ref().unwrap().send(()).unwrap();
        // }

        let mut logs_state = self.logs_state.borrow_mut();
        ui!((area, buf) => {
            Stateful(&mut logs_state) {
                { self.logs }
            }
        });
    }
}
