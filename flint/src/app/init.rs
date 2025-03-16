use super::AppWidget;
use crate::util::{handle_key_events, lang::Language, toml::Config};
use clap::Parser;
use crossterm::event::{Event, KeyCode};
use flint_macros::{ui, widget as w};
use flint_utils::{get_flag, info, Error, Result};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Paragraph, WidgetRef, Wrap},
};
use std::{collections::BTreeSet, path::PathBuf};
use tui_textarea::TextArea;

#[derive(Debug)]
#[allow(unused)]
pub struct InitWidget<'a> {
    textarea: TextArea<'a>,
    langs: BTreeSet<Language>,
    created_config: bool,
    config_exists: bool,
    args: InitWidgetArgs,
}

#[derive(Parser, Debug, Clone)]
pub struct InitWidgetArgs {
    /// Show help for the init command
    #[clap(short, long)]
    help: bool,
}

impl<'a> InitWidget<'a> {
    pub fn new(args: InitWidgetArgs) -> Self {
        Self {
            textarea: TextArea::default(),
            langs: BTreeSet::new(),
            created_config: false,
            config_exists: false,
            args,
        }
    }
}

impl<'a> WidgetRef for InitWidget<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let confirm_message = if self.config_exists {
            "flint.toml already exists in this directory. Would you like to overwrite it? (y/n)"
        } else {
            "Would you like to continue with creating flint.toml? (y/n)"
        };

        let textarea = w!({
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

        let languages = self
            .langs
            .iter()
            .map(|lang| match lang {
                Language::Supported(name) => Line::from(name.clone()),
                Language::Unsupported(name) => {
                    Line::from(name.clone()).style(Style::default().fg(Color::Red).bold())
                }
            })
            .collect::<Vec<Line>>();

        ui!((area, buf) => {
            Layout(
                constraints: [Constraint::Fill(1)],
                direction: Direction::Horizontal
            ) {
                Layout (
                    direction: Direction::Vertical,
                    constraints: [Constraint::Length((self.langs.len() + 2) as u16), Constraint::Min(1)]
                ) {
                    Paragraph::new(
                      languages,
                      block: w!({ Block::bordered(title: "We found the following languages in this directory (".to_string() + &self.langs.iter().filter(|lang| matches!(lang, Language::Unsupported(_))).count().to_string() + " unsupported)") }),
                      wrap: Wrap { trim: false }
                    ),
                    If (!self.created_config) {
                        {textarea}
                    } Else {
                        Paragraph::new("Configuration created successfully. Press any key to exit.", style: w!({ Style(fg: Color::Green) }))
                    }
                },
            }
        }
        );
    }
}

impl<'a> AppWidget for InitWidget<'a> {
    fn setup(&mut self) -> Result<()> {
        let cwd = get_flag!(current_dir);
        info!("Determined current directory: {}", cwd.display());

        self.langs = crate::util::detect_languages(cwd.to_str().unwrap());

        let config_path = std::path::Path::new(&cwd).join("flint.toml");
        if config_path.exists() {
            self.config_exists = true;
        }

        Ok(())
    }

    fn handle_events(&mut self, event: Event) -> Result<()> {
        handle_key_events(event, |key_event, key_code| {
            if self.created_config {
                return Err(Error::Exit);
            }
            match key_code {
                KeyCode::Enter => {
                    let input = self.textarea.lines().get(0).unwrap();

                    match input.as_str() {
                        "n" => return Err(Error::Exit),
                        "y" => {
                            Config::create_default(PathBuf::from("./flint.toml")).unwrap();
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
