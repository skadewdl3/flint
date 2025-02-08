use crossterm::event::Event;
use ratatui::Frame;
use std::io;

pub mod app;
pub mod generate;
pub mod help;
pub mod init;
pub mod test;

pub use app::*;

pub trait AppWidget {
    fn setup(&mut self) {}
    fn draw(&mut self, frame: &mut Frame);
    fn handle_events(&mut self, event: Event) -> io::Result<()> {
        let _ = event;
        Ok(())
    }
}
