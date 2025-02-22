use crossterm::event::Event;
use ratatui::widgets::WidgetRef;

pub mod app;
pub mod generate;
pub mod help;
pub mod init;
pub mod test;
pub use app::*;
pub mod logs;

pub trait AppWidget: WidgetRef {
    fn setup(&mut self) -> AppStatus {
        return AppStatus::Ok;
    }
    fn handle_events(&mut self, _event: Event) -> AppStatus {
        return AppStatus::Ok;
    }
}

#[derive(Copy, Clone)]
pub enum AppStatus<'a> {
    Ok,
    Exit,
    Error(&'a str),
}

impl AppStatus<'_> {
    pub fn into_result(&self) -> Result<(), std::io::Error> {
        match self {
            Self::Ok => Ok(()),
            Self::Exit => Ok(()),
            Self::Error(msg) => Err(std::io::Error::new(std::io::ErrorKind::Other, *msg)),
        }
    }
}
