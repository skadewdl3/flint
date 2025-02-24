use super::generate::GenerateWidget;
use super::help::HelpWidget;
use super::init::InitWidget;
use super::test::TestWidget;
use super::AppWidget;
use crate::util::handle_key_events;
use crate::widgets::logs::{add_log, LogKind};
use crossterm::event;
use crossterm::event::KeyCode;
use flint_macros::{ui, widget};
use ratatui::widgets::WidgetRef;
use ratatui::{prelude::*, DefaultTerminal};
use std::io;
use std::time::Duration;
use tui_popup::Popup;

use super::{AppError, AppResult};

pub struct App {
    exit: bool,
    active_widget: Box<dyn AppWidget>,
    error: Option<String>,
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
        let args: Vec<String> = std::env::args().collect();

        if let Some(arg) = args.get(1) {
            self.active_widget = match arg.as_str() {
                "init" => Box::new(InitWidget::default()),
                "generate" => Box::new(GenerateWidget::default()),
                "test" => Box::new(TestWidget::default()),
                _ => Box::new(HelpWidget::default()),
            };
        } else {
            self.active_widget = Box::new(HelpWidget::default());
        }

        match self.active_widget.setup() {
            Ok(_) => (),
            Err(err) => {
                self.error = Some(err.to_string());
                self.exit = true;
            }
        }
        while !self.exit {
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
