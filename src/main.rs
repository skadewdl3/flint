mod generate;
mod help;
mod init;
mod test;
pub mod util;

use std::{io, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use generate::GenerateWidget;
use help::HelpWidget;
use init::InitWidget;
use ratatui::{DefaultTerminal, Frame};
use test::TestWidget;

pub trait AppWidget {
    fn draw(&mut self, frame: &mut Frame);
    fn handle_events(&mut self, event: Event) -> io::Result<()> {
        let _ = event;
        Ok(())
    }
}

struct App {
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
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Char('q') => self.exit = true,
                    _ => {}
                }
            }
            _ => {}
        }
        self.active_widget.handle_events(event)?;

        Ok(())
    }
}

fn main() {
    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal);
    app_result.expect("Error while running app");
    ratatui::restore();
}
