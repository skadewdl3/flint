use std::io;

use crossterm::event::{read, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::Text,
    widgets::{Block, List, ListDirection, ListState},
    Frame,
};
use throbber_widgets_tui::{Throbber, ThrobberState};
use tui_textarea::TextArea;

use crate::handle_key_event;

use super::AppWidget;

#[derive(Debug)]
pub struct InitWidget<'a> {
    list_state: ListState,
    textarea: TextArea<'a>,
}

impl<'a> Default for InitWidget<'a> {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            list_state,
            textarea: TextArea::default(),
        }
    }
}

impl<'a> AppWidget for InitWidget<'a> {
    fn draw(&mut self, frame: &mut Frame) {
        let file_path = match std::env::current_dir() {
            Ok(path) => path,
            Err(_) => panic!("Unable to determine current directory"),
        };

        let file_path = file_path.to_str().unwrap();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(frame.area());

        frame.render_widget(
            Text::raw("We found the following languages in this directory."),
            layout[0],
        );

        let detected_langs = crate::util::file::detect_languages(file_path);
        let list: Vec<String> = detected_langs.into_iter().collect();

        let list_widget = List::new(list)
            .block(Block::bordered().title("Detected Languages"))
            .repeat_highlight_symbol(true);

        frame.render_stateful_widget(list_widget, layout[1], &mut self.list_state);

        frame.render_widget(Text::raw("Would you like to create flint.toml?"), layout[2]);

        frame.render_widget(&self.textarea, layout[3]);
    }

    fn handle_events(&mut self, event: Event) -> io::Result<()> {
        if let Event::Key(key) = event {
            self.textarea.input(key);
        }
        Ok(())
    }
}
