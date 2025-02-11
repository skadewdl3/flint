use ratatui::Frame;

use super::{AppStatus, AppWidget};

#[derive(Debug)]
pub struct HelpWidget {}

impl Default for HelpWidget {
    fn default() -> Self {
        Self {}
    }
}

impl AppWidget for HelpWidget {
    fn draw(&mut self, frame: &mut Frame) -> AppStatus {
        AppStatus::Ok
    }
}
