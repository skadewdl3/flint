use crate::util::handle_key_events;
use crate::widgets::generate::GenerateWidget;
use crate::widgets::help::HelpWidget;
use crate::widgets::init::InitWidget;
use crate::widgets::test::TestWidget;
use crate::widgets::AppWidget;
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use ratatui::DefaultTerminal;
use ratatui::Frame;
use std::io;
use std::time::Duration;
use tui_popup::Popup;

use super::AppStatus;

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
        }

        match self.active_widget.setup() {
            AppStatus::Error(err) => {
                self.error = Some(err.to_string());
            }
            AppStatus::Exit => {
                self.exit = true;
            }
            _ => (),
        }
        while !self.exit {
            terminal.draw(|frame| match self.draw(frame) {
                AppStatus::Exit => self.exit = true,
                AppStatus::Error(err) => {
                    self.error = Some(err.to_string());
                }
                _ => (),
            })?;

            let status = self.handle_all_events();
            match status {
                AppStatus::Exit => self.exit = true,
                AppStatus::Error(err) => {
                    self.error = Some(err.to_string());
                }
                _ => (),
            }
        }

        Ok(())
    }

    fn handle_all_events(&mut self) -> AppStatus {
        // Exit early if no events are available
        if let Ok(event_exists) = event::poll(Duration::from_millis(100)) {
            if !event_exists {
                return AppStatus::Ok;
            }
        }

        let event = event::read().expect("Could not get event");
        self.handle_events(event)
    }
}

impl AppWidget for App {
    fn draw(&mut self, frame: &mut Frame) -> AppStatus {
        let status = self.active_widget.draw(frame);
        if let Some(err) = &self.error {
            let popup = Popup::new(err.as_str()).title(format!("Error - {}", err));
            frame.render_widget(&popup, frame.area());
        }
        status
    }

    fn handle_events(&mut self, event: Event) -> AppStatus {
        let status = handle_key_events(event.clone(), |_, key_code| {
            match key_code {
                KeyCode::Esc => return AppStatus::Exit,
                _ => {}
            }
            AppStatus::Ok
        });
        if let AppStatus::Exit = status {
            return AppStatus::Exit;
        }

        let status1 = self.active_widget.handle_events(event);
        status1
    }
}
