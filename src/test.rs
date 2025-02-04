use ratatui::Frame;
use throbber_widgets_tui::{Throbber, ThrobberState};

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

impl crate::AppWidget for TestWidget {
    fn draw(&mut self, frame: &mut Frame) {
        self.throbber_state.calc_next();
        let throbber = Self::get_throbber("Loading Test Window");
        frame.render_stateful_widget(throbber, frame.area(), &mut self.throbber_state);
    }
}

impl TestWidget {
    pub fn get_throbber<'a>(label: &'a str) -> Throbber<'a> {
        Throbber::default()
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan))
            .label(label)
            .throbber_style(
                ratatui::style::Style::default()
                    .fg(ratatui::style::Color::Red)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .throbber_set(throbber_widgets_tui::CLOCK)
            .use_type(throbber_widgets_tui::WhichUse::Spin)
    }
}
