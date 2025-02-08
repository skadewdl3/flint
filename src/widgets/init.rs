use super::AppWidget;
use crate::util::handle_key_events;
use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout},
    style::{Color, Stylize},
    text::Text,
    widgets::{Block, List, ListState},
    Frame,
};
use std::io;
use tui_textarea::TextArea;

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

        let layout0 = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Fill(1)])
            .split(frame.area());

        frame.render_widget(
            Text::raw("We found the following languages in this directory."),
            layout0[0],
        );

        let (detected_langs, unsupported_langs) = crate::util::file::detect_languages(file_path);
        let layout1 = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length((detected_langs.len() + 2) as u16),
                Constraint::Length((unsupported_langs.len() + 2) as u16),
                Constraint::Length(1),
            ])
            .split(layout0[1]);

        let detected_langs: Vec<String> = detected_langs.into_iter().collect();
        let list_widget = List::new(detected_langs)
            .block(Block::bordered().title("Detected Languages"))
            .repeat_highlight_symbol(true);
        frame.render_widget(list_widget, layout1[0]);

        if unsupported_langs.len() > 0 {
            let unsupported_langs: Vec<String> = unsupported_langs.into_iter().collect();
            let unsupported_list_widget = List::new(unsupported_langs)
                .block(Block::bordered().title("Unsupported Languages"))
                .repeat_highlight_symbol(true);
            frame.render_widget(unsupported_list_widget, layout1[1]);
        }

        let layout2 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(58),
                Constraint::Length(2),
                Constraint::Fill(1),
            ])
            .flex(Flex::Center)
            .split(layout1[2]);

        frame.render_widget(
            Block::default()
                .title("Would you like to continue with creating flint.toml? (y/n)")
                .fg(Color::Yellow),
            layout2[0],
        );

        frame.render_widget(&self.textarea, layout2[2]);
    }

    fn handle_events(&mut self, event: Event) -> io::Result<()> {
        handle_key_events(&event, |key_event, key_code| match key_code {
            KeyCode::Enter => {
                println!("Got text");
            }
            _ => {
                self.textarea.input(key_event);
            }
        });
        Ok(())
    }
}
