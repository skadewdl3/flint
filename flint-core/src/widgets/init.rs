use super::{AppStatus, AppWidget};
use crate::util::{
    handle_key_events,
    toml::{create_toml_config, Config, FlintConfig},
};
use crossterm::event::{Event, KeyCode};
use flint_macros::{ui, widget};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    text::Text,
    widgets::{Block, List, Paragraph, WidgetRef},
};
use std::collections::{BTreeSet, HashMap};
use tui_textarea::TextArea;

#[derive(Debug, Default)]
pub struct InitWidget<'a> {
    textarea: TextArea<'a>,
    detected_langs: BTreeSet<String>,
    unsupported_langs: BTreeSet<String>,
    _created_config: bool,
    config_exists: bool,
    cwd: String,
}

impl<'a> InitWidget<'a> {
    pub fn hello_world(&self) -> &'a str {
        "Hello World"
    }
}

impl<'a> WidgetRef for InitWidget<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let confirm_message = if self.config_exists {
            "flint.toml already exists in this directory. Would you like to overwrite it? (y/n)"
        } else {
            "Would you like to continue with creating flint.toml? (y/n)"
        };

        ui!((area, buf) => {
            Layout (
                direction: Direction::Vertical,
                constraints: Constraint::from_lengths([

                    self.detected_langs.len() as u16 + 2,
                    self.unsupported_langs.len() as u16 + 2,
                    1
                ])
            ){
                List::new(
                    self.detected_langs.clone(),
                    block: widget!({ Block::bordered(title: "Detected Languages") })
                ),

                List::new(
                                    self.unsupported_langs.clone(),
                                    block: widget!({ Block::bordered(title: "Detected Languages") })
                                )
            }
        }
        );
    }
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
