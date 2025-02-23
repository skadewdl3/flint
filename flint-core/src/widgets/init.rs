use super::{
    logs::{add_log, LogKind, LogsWidget},
    AppError, AppResult, AppWidget,
};
use crate::util::{
    handle_key_events,
    toml::{create_toml_config, Config, FlintConfig},
};
use crossterm::event::{Event, KeyCode};
use flint_macros::{ui, widget};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, List, Paragraph, WidgetRef},
};
use std::{
    collections::{BTreeSet, HashMap},
    path::PathBuf,
};
use tui_textarea::TextArea;

#[derive(Debug, Default)]
pub struct InitWidget<'a> {
    textarea: TextArea<'a>,
    detected_langs: BTreeSet<String>,
    unsupported_langs: BTreeSet<String>,
    created_config: bool,
    config_exists: bool,
    cwd: PathBuf,
    logs: LogsWidget,
}

impl<'a> WidgetRef for InitWidget<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let confirm_message = if self.config_exists {
            "flint.toml already exists in this directory. Would you like to overwrite it? (y/n)"
        } else {
            "Would you like to continue with creating flint.toml? (y/n)"
        };

        let textarea = widget!({
            Layout(
                direction: Direction::Horizontal,
                constraints: [
                    Constraint::Length(confirm_message.len() as u16),
                    Constraint::Length(1),
                    Constraint::Fill(1)
                ]
            ) {
                Paragraph::new(confirm_message, style: Style::default().fg(Color::Yellow)),
                {" "},
                {&self.textarea}
            }
        });

        ui!((area, buf) => {
            Layout(
                constraints: [Constraint::Fill(1), Constraint::Length(2), Constraint::Fill(1)],
                direction: Direction::Horizontal
            ) {

                Layout (
                    direction: Direction::Vertical,
                    constraints: Constraint::from_lengths([
                        1,
                        self.detected_langs.len() as u16 + 2,
                        self.unsupported_langs.len() as u16 + 2,
                        1
                    ])
                ) {
                    {"We found the following languages in this directory."},

                    List::new(
                        self.detected_langs.clone(),
                        block: widget!({ Block::bordered(title: "Detected Languages") }),
                    ),
                    List::new(
                        self.unsupported_langs.clone(),
                        block: widget!({ Block::bordered(title: "Unsupported Languages") }),
                    ),
                    If (!self.created_config) {
                        {textarea}
                    } Else {
                        Paragraph::new("Configuration created successfully")
                    }
                },
                {""},
                {self.logs}
            }
        }
        );
    }
}

impl<'a> AppWidget for InitWidget<'a> {
    fn setup(&mut self) -> AppResult<()> {
        self.cwd = std::env::current_dir()?;
        add_log(
            LogKind::Info,
            format!("Determined current directory: {}", self.cwd.display()),
        );

        let (detected_langs, unsupported_langs) =
            crate::util::detect_languages(self.cwd.to_str().unwrap());
        self.detected_langs = detected_langs;
        self.unsupported_langs = unsupported_langs;

        let config_path = std::path::Path::new(&self.cwd).join("flint.toml");
        if config_path.exists() {
            self.config_exists = true;
        }

        Ok(())
    }

    fn handle_events(&mut self, event: Event) -> AppResult<()> {
        handle_key_events(event, |key_event, key_code| {
            match key_code {
                KeyCode::Enter => {
                    let input = self.textarea.lines().get(0).unwrap();

                    match input.as_str() {
                        "n" => return Err(AppError::Exit),
                        "y" => {
                            let config = Config {
                                flint: FlintConfig { version: 1 },
                                linters: HashMap::new(),
                                common: HashMap::new(),
                            };

                            create_toml_config("./flint.toml", config)?;
                            self.created_config = true;
                        }
                        _ => (),
                    }
                }
                _ => {
                    self.textarea.input(key_event);
                }
            }
            Ok(())
        })
    }
}
