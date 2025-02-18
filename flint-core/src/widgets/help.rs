use ratatui::{widgets::WidgetRef, Frame};

use super::{AppStatus, AppWidget};

#[derive(Debug)]
pub struct HelpWidget {}

impl Default for HelpWidget {
    fn default() -> Self {
        Self {}
    }
}

impl WidgetRef for HelpWidget {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {}
}
impl AppWidget for HelpWidget {}
