use std::sync::{OnceLock, RwLock, RwLockReadGuard};

use flint_macros::ui;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{List, Paragraph, Widget, Wrap},
    Frame,
};

use super::{AppStatus, AppWidget};

#[derive(Copy, Clone, Debug, Default)]
pub enum LogKind {
    #[default]
    Info,
    Success,
    Error,
    Warn,
    Debug,
}

pub static LOGS: OnceLock<RwLock<Vec<(LogKind, String)>>> = OnceLock::new();

#[derive(Debug, Default, Copy, Clone)]
pub struct LogsWidget;

pub fn get_logs() -> Result<
    RwLockReadGuard<'static, Vec<(LogKind, String)>>,
    std::sync::PoisonError<RwLockReadGuard<'static, Vec<(LogKind, String)>>>,
> {
    let x = LOGS.get_or_init(|| RwLock::new(vec![])).read();
    x
}

pub fn clear_logs() {
    if let Some(logs) = LOGS.get() {
        logs.write().unwrap().clear();
    }
}

pub fn add_log(kind: LogKind, message: String) {
    if let Some(logs) = LOGS.get() {
        logs.write().unwrap().push((kind, message));
    }
}

impl Widget for LogsWidget {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let logs = get_logs().unwrap();

        let log_lines: Vec<u16> = logs
            .iter()
            .map(|(kind, log)| match kind {
                LogKind::Debug => log.lines().count() as u16 + 1,
                _ => log.lines().count() as u16,
            })
            .collect();

        // let layout = Layout::default()
        //     .direction(Direction::Vertical)
        //     .constraints(Constraint::from_lengths(log_lines))
        //     .split(frame.area());

        // for i in 0..logs.len() {
        //     frame.render_widget(text, layout[i]);
        // }
        //

        ui!((area, buffer) =>
            Layout(
                direction: Direction::Vertical,
                constraints: Constraint::from_lengths(log_lines),
            ) {
                [[
                    logs.iter().map(|(kind, log)| {

                        let prefix = match kind {
                          LogKind::Info => "[info]:",
                          LogKind::Success => "[success]:",
                          LogKind::Error => "[error]:",
                          LogKind::Warn => "[warn]:",
                          LogKind::Debug => "[debug]:"
                        };


                        let style = match kind {
                            LogKind::Info => Style::default().fg(Color::Blue),
                            LogKind::Success => Style::default().fg(Color::Green),
                            LogKind::Error => Style::default().fg(Color::Red),
                            LogKind::Warn => Style::default().fg(Color::Yellow),
                            LogKind::Debug => Style::default().fg(Color::White)
                        };

                        Paragraph::new(format!("{} {}", prefix, log)).wrap(Wrap{trim: true}).style(style)
                    })
                ]]
            }
        );
    }
}

impl AppWidget for LogsWidget {
    fn draw(&mut self, frame: &mut Frame) -> AppStatus {
        AppStatus::Ok
    }
}
