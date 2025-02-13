use ratatui::Frame;
use throbber_widgets_tui::{Throbber, ThrobberState};

use super::{AppStatus, AppWidget};

#[derive(Debug)]
pub struct TestWidget {
    throbber_state: ThrobberState,
}

impl Default for TestWidget {
    fn default() -> Self {
        Self {
            throbber_state: ThrobberState::default(),
        }
    }
}

impl AppWidget for TestWidget {
    fn draw(&mut self, frame: &mut Frame) -> AppStatus {
        AppStatus::Ok
    }
}
