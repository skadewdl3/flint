use ratatui::{widgets::WidgetRef, Frame};
use super::{AppStatus, AppWidget};
use ratatui::prelude::*;
use ratatui::{widgets::WidgetRef, Frame};

#[derive(Debug)]
pub struct HelpWidget {}

impl Default for HelpWidget {
    fn default() -> Self {
        Self {}
    }
}

impl AppWidget for HelpWidget {}

impl WidgetRef for HelpWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {}
}
impl AppWidget for HelpWidget {}
