use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::Text,
    widgets::{List, ListState},
    Frame,
};
use throbber_widgets_tui::{Throbber, ThrobberState};

use crate::util::toml::create_toml_config;

#[derive(Debug)]
pub struct InitWidget {
    throbber_state: ThrobberState,
    list_state: ListState,
}

impl Default for InitWidget {
    fn default() -> Self {
        Self {
            throbber_state: ThrobberState::default(),
            list_state: ListState::default(),
        }
    }
}

impl InitWidget {
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

    pub fn draw(&mut self, frame: &mut Frame) {
        self.throbber_state.calc_next();
        let throbber = Self::get_throbber("Loading Init Window");

        let file_path = match std::env::current_dir() {
            Ok(path) => path,
            Err(_) => panic!("Unable to determine current directory"),
        };

        let file_path = file_path.to_str().unwrap();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(Constraint::from_lengths([1, 2, 4]))
            .split(frame.area());

        frame.render_stateful_widget(throbber, layout[0], &mut self.throbber_state);
        frame.render_widget(
            Text::raw(format!("Current Directory: {}", file_path)),
            layout[1],
        );

        let detected_langs = crate::util::file::detect_languages(file_path);
        let list: Vec<String> = detected_langs.into_iter().collect();

        let list_widget = List::new(list);
        frame.render_stateful_widget(list_widget, layout[2], &mut self.list_state);

        _ = create_toml_config("./flint.toml");
    }
}
