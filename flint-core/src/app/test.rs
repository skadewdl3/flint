use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;
use throbber_widgets_tui::ThrobberState;

use super::AppWidget;

#[derive(Debug)]
pub struct TestWidget {
    _throbber_state: ThrobberState,
}

impl Default for TestWidget {
    fn default() -> Self {
        Self {
            _throbber_state: ThrobberState::default(),
        }
    }
}

impl AppWidget for TestWidget {}
impl WidgetRef for TestWidget {
    fn render_ref(&self, _area: Rect, _buf: &mut Buffer) {}
}
