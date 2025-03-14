use super::generate::{GenerateWidget, GenerateWidgetArgs};
use super::help::HelpWidget;
use super::init::{InitWidget, InitWidgetArgs};
use super::install::{InstallArgs, InstallWidget};
use super::test::{TestArgs, TestWidget};
use super::AppWidget;
use crate::util::handle_key_events;
use clap::{Parser, Subcommand};
use crossterm::event;
use crossterm::event::KeyCode;
use flint_macros::{ui, widget};
use flint_utils::{error, Error, Result};
use ratatui::widgets::WidgetRef;
use ratatui::{prelude::*, DefaultTerminal};
use std::io;
use std::sync::mpsc;
use std::time::Duration;
use threadpool::ThreadPool;
use tui_popup::Popup;

pub struct App {
    exit: bool,
    active_widget: Box<dyn AppWidget>,
    error: Option<String>,
    sender: mpsc::Sender<()>,
    receiver: mpsc::Receiver<()>,
    args: AppArgs,
}

#[derive(Parser, Clone)]
#[command(version, about, long_about = None, disable_help_subcommand = true, disable_help_flag = true)]
pub struct AppArgs {
    #[clap(long, global = false)]
    pub plugins_dir: Option<String>,

    #[clap(long, global = false)]
    pub config_path: Option<String>,

    #[clap(long, default_value_t = false, global = false)]
    pub no_install: bool,

    #[command(subcommand)]
    pub command: Option<AppWidgetArgs>,
}

#[derive(Subcommand, Clone)]
#[command(version, about, long_about = None, disable_help_subcommand = true, disable_help_flag = true)]
pub enum AppWidgetArgs {
    /// Initializes a flint.toml file
    Init(InitWidgetArgs),
    /// Generates linter and testing library configuration files
    Generate(GenerateWidgetArgs),
    /// Tests a flint project
    Test(TestArgs),
    /// Installs the given list of plugins
    Install(InstallArgs),
    Help,
}

impl App {
    pub fn new(args: AppArgs) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            exit: false,
            active_widget: Box::new(HelpWidget::default()),
            error: None,
            sender,
            receiver,
            args,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let args = self.args.clone();

        self.active_widget = match args.command.unwrap() {
            AppWidgetArgs::Install(args) => Box::new(InstallWidget::new(args)),
            AppWidgetArgs::Generate(args) => Box::new(GenerateWidget::new(args)),
            AppWidgetArgs::Test(args) => Box::new(TestWidget::new(args)),
            AppWidgetArgs::Init(args) => Box::new(InitWidget::new(args)),
            _ => Box::new(HelpWidget::default()),
        };

        self.active_widget.set_exit_sender(self.sender.clone());

        let thread_pool = ThreadPool::new(16);
        self.active_widget.set_thread_pool(&thread_pool);

        match self.active_widget.setup() {
            Ok(_) => (),
            Err(err) => {
                self.error = Some(err.to_string());
                self.exit = true;
            }
        }
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            if let Ok(_) = self.receiver.recv_timeout(Duration::from_millis(1)) {
                // Break if a forceful exit is requested
                break;
            }

            match self.handle_all_events() {
                Ok(_) => (),
                Err(err) => {
                    self.error = Some(err.to_string());
                    self.exit = true;
                }
            }
        }

        thread_pool.join();

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        ui!(frame => {
            { self }
        });
    }

    fn handle_all_events(&mut self) -> Result<()> {
        // Exit early if no events are available
        if let Ok(event_exists) = event::poll(Duration::from_millis(100)) {
            if !event_exists {
                return Ok(());
            }
        }

        let event = event::read().expect("Could not get event");
        let status1 = handle_key_events(event.clone(), |_, key_code| match key_code {
            KeyCode::Esc => return Err(Error::Exit),
            _ => Ok(()),
        });

        let status2 = self.active_widget.handle_events(event);
        if matches!(status1, Err(Error::Exit)) || matches!(status2, Err(Error::Exit)) {
            return Err(Error::Exit);
        }
        Ok(())
    }
}

impl WidgetRef for App {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        ui!((area, buf) => {
            {
                &self.error.as_ref().map(|err| {
                    error!("Error occurred: {}", err);
                    Some(widget!({ Popup::new(err.as_str(), title: format!("Error occurred")) }))
                })
            }
        });

        self.active_widget.render_ref(area, buf);
    }
}
