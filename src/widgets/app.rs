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

pub struct App {
    exit: bool,
    active_widget: Box<dyn AppWidget>,
}

impl App {
    pub fn new() -> Self {
        Self {
            exit: false,
            active_widget: Box::new(HelpWidget::default()),
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

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_all_events()?;
        }

        Ok(())
    }

    fn handle_all_events(&mut self) -> io::Result<()> {
        // Exit early if no events are available
        if let Ok(event_exists) = event::poll(Duration::from_millis(100)) {
            if !event_exists {
                return Ok(());
            }
        }

        let event = event::read().expect("Could not get event");
        self.handle_events(event)
    }
}

impl AppWidget for App {
    fn draw(&mut self, frame: &mut Frame) {
        self.active_widget.draw(frame);
    }

    fn handle_events(&mut self, event: Event) -> io::Result<()> {
        handle_key_events(&event, |_, key_code| match key_code {
            KeyCode::Esc => self.exit = true,
            _ => {}
        });

        self.active_widget.handle_events(event)?;
        Ok(())
    }
}
