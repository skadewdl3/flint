use super::generate::{GenerateWidget, GenerateWidgetArgs};
use super::help::HelpWidget;
use super::init::{InitWidget, InitWidgetArgs};
use super::install::{InstallArgs, InstallWidget};
use super::test::{TestArgs, TestWidget};
use super::AppWidget;
use super::{AppError, AppResult};
use crate::util::handle_key_events;
use crate::widgets::logs::{add_log, LogKind};
use clap::{Parser, Subcommand};
use crossterm::event;
use crossterm::event::KeyCode;
use flint_macros::{ui, widget};
use ratatui::widgets::WidgetRef;
use ratatui::{prelude::*, DefaultTerminal};
use std::io;
use std::time::Duration;
use tui_popup::Popup;

pub struct App {
    exit: bool,
    active_widget: Box<dyn AppWidget>,
    error: Option<String>,
}

#[derive(Parser)]
#[command(version, about, long_about = None, disable_help_subcommand = true, disable_help_flag = true)]
struct Args {
    #[command(subcommand)]
    command: Option<AppWidgetArgs>,
    // #[clap(short, long)]
    // help: bool,
}

#[derive(Subcommand)]
#[command(version, about, long_about = None, disable_help_subcommand = true, disable_help_flag = true)]
enum AppWidgetArgs {
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
    pub fn new() -> Self {
        Self {
            exit: false,
            active_widget: Box::new(HelpWidget::default()),
            error: None,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut args: Vec<String> = std::env::args().collect();

        // Check for --headless and remove it from the args list
        let headless = if let Some(index) = args.iter().position(|arg| arg == "--headless") {
            args.remove(index);
            true
        } else {
            false
        };

        // Parse the command from modified args list
        let args = Args::parse_from(args);

        self.active_widget = match args.command.unwrap() {
            AppWidgetArgs::Install(args) => Box::new(InstallWidget::new(args)),
            AppWidgetArgs::Generate(args) => Box::new(GenerateWidget::new(args)),
            AppWidgetArgs::Test(args) => Box::new(TestWidget::new(args)),
            AppWidgetArgs::Init(args) => Box::new(InitWidget::new(args)),
            _ => Box::new(HelpWidget::default()),
        };

        match self.active_widget.setup() {
            Ok(_) => (),
            Err(err) => {
                self.error = Some(err.to_string());
                self.exit = true;
            }
        }
        while !self.exit && !headless {
            terminal.draw(|frame| self.draw(frame))?;

            match self.handle_all_events() {
                Ok(_) => (),
                Err(err) => {
                    self.error = Some(err.to_string());
                    self.exit = true;
                }
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        ui!(frame => {
            { self }
        });
    }

    fn handle_all_events(&mut self) -> AppResult<()> {
        // Exit early if no events are available
        if let Ok(event_exists) = event::poll(Duration::from_millis(100)) {
            if !event_exists {
                return Ok(());
            }
        }

        let event = event::read().expect("Could not get event");
        let status1 = handle_key_events(event.clone(), |_, key_code| match key_code {
            KeyCode::Esc => return Err(AppError::Exit),
            _ => Ok(()),
        });

        let status2 = self.active_widget.handle_events(event);
        if matches!(status1, Err(AppError::Exit)) || matches!(status2, Err(AppError::Exit)) {
            return Err(AppError::Exit);
        }
        Ok(())
    }
}

impl WidgetRef for App {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        ui!((area, buf) => {
            {
                &self.error.as_ref().map(|err| {
                    add_log(LogKind::Error, err.clone());
                    Some(widget!({ Popup::new(err.as_str(), title: format!("Error occurred")) }))
                })
            }
        });

        self.active_widget.render_ref(area, buf);
    }
}
