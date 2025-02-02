mod generate;
mod help;
mod init;
mod test;
pub mod util;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use generate::GenerateWidget;
use help::HelpWidget;
use init::InitWidget;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::{Line, Text},
    widgets::{Paragraph, Widget},
    DefaultTerminal, Frame,
};
use std::{env, io, time::Duration};
use test::TestWidget;

#[derive(Debug, Default)]
enum AppMode {
    #[default]
    Help,
    Init,
    Generate,
    Test,
}

#[derive(Debug, Default)]
pub struct App {
    mode: AppMode,
    counter: u8,
    exit: bool,
    init_widget: InitWidget,
    generate_widget: GenerateWidget,
    help_widget: HelpWidget,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let args = env::args().collect::<Vec<String>>();
        if let Some(mode) = args.get(1) {
            self.mode = match mode.as_str() {
                "generate" => AppMode::Generate,
                "init" => AppMode::Init,
                "test" => AppMode::Test,
                _ => AppMode::Help,
            };
        } else {
            self.mode = AppMode::Help;
        }

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        match self.mode {
            AppMode::Generate => self.generate_widget.draw(frame),
            AppMode::Init => self.init_widget.draw(frame),
            AppMode::Test => TestWidget::default().draw(frame),
            _ => self.help_widget.draw(frame),
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        // return early if no events are available
        if let Ok(event_available) = event::poll(Duration::from_millis(100)) {
            if !event_available {
                return Ok(());
            }
        }

        // If event is available, read and handle it
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text).centered().render(area, buf);
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

fn init() {}

fn usage() {}
