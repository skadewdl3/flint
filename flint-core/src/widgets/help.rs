use super::AppWidget;
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;

#[derive(Debug)]
pub struct HelpWidget {}

impl Default for HelpWidget {
    fn default() -> Self {
        Self {}
    }
}

impl AppWidget for HelpWidget {}

impl WidgetRef for HelpWidget {
    fn render_ref(&self, _area: Rect, _buf: &mut Buffer) {}
}
