use super::{AppStatus, AppWidget};
use crate::util::{
    handle_key_events,
    toml::{create_toml_config, Config, FlintConfig},
};
use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout},
    style::{Color, Stylize},
    text::Text,
    widgets::{Block, List},
    Frame,
};
use std::collections::{BTreeSet, HashMap};
use tui_textarea::TextArea;

#[derive(Debug, Default)]
pub struct InitWidget<'a> {
    textarea: TextArea<'a>,
    detected_langs: BTreeSet<String>,
    unsupported_langs: BTreeSet<String>,
    created_config: bool,
    config_exists: bool,
    cwd: String,
}

impl<'a> AppWidget for InitWidget<'a> {
    fn setup(&mut self) -> AppStatus {
        let file_path = match std::env::current_dir() {
            Ok(path) => path,
            Err(_) => return AppStatus::Error("Unable to determine current directory"),
        };

        let file_path = file_path
            .to_str()
            .expect("Unable to convert path to string");

        let (detected_langs, unsupported_langs) = crate::util::detect_languages(file_path);
        self.detected_langs = detected_langs;
        self.unsupported_langs = unsupported_langs;
        self.cwd = file_path.to_string();

        let config_path = std::path::Path::new(&self.cwd).join("flint.toml");
        if config_path.exists() {
            self.config_exists = true;
        }

        AppStatus::Ok
    }

    fn draw(&mut self, frame: &mut Frame) -> AppStatus {
        let layout0 = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Fill(1)])
            .split(frame.area());

        frame.render_widget(
            Text::raw("We found the following languages in this directory."),
            layout0[0],
        );

        let layout1 = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length((self.detected_langs.len() + 2) as u16),
                Constraint::Length((self.unsupported_langs.len() + 2) as u16),
                Constraint::Length(1),
            ])
            .split(layout0[1]);

        let list_widget = List::new(self.detected_langs.clone())
            .block(Block::bordered().title("Detected Languages"))
            .repeat_highlight_symbol(true);
        frame.render_widget(list_widget, layout1[0]);

        if self.unsupported_langs.len() > 0 {
            let unsupported_list_widget = List::new(self.unsupported_langs.clone())
                .block(Block::bordered().title("Unsupported Languages"))
                .repeat_highlight_symbol(true);
            frame.render_widget(unsupported_list_widget, layout1[1]);
        }

        let confirm_message = if self.config_exists {
            "flint.toml already exists in this directory. Would you like to overwrite it? (y/n)"
        } else {
            "Would you like to continue with creating flint.toml? (y/n)"
        };

        let layout2 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(confirm_message.len() as u16),
                Constraint::Length(2),
                Constraint::Fill(1),
            ])
            .flex(Flex::Center)
            .split(layout1[2]);

        frame.render_widget(
            Block::default().title(confirm_message).fg(Color::Yellow),
            layout2[0],
        );

        frame.render_widget(&self.textarea, layout2[2]);
        AppStatus::Ok
    }

    fn handle_events(&mut self, event: Event) -> AppStatus {
        handle_key_events(event, |key_event, key_code| {
            match key_code {
                KeyCode::Enter => {
                    let input = self.textarea.lines().get(0).unwrap();

                    match input.as_str() {
                        "n" => return AppStatus::Exit,
                        "y" => {
                            let config = Config {
                                flint: FlintConfig { version: 1 },
                                linters: HashMap::new(),
                                common: HashMap::new(),
                            };

                            create_toml_config("./flint.toml", config).unwrap();
                        }
                        _ => (),
                    }
                }
                _ => {
                    self.textarea.input(key_event);
                }
            }
            AppStatus::Ok
        })
    }
}
