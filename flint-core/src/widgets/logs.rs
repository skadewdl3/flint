use std::sync::{OnceLock, RwLock, RwLockReadGuard};

use flint_macros::{ui, widget};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Widget, WidgetRef},
};

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

impl WidgetRef for LogsWidget {
    fn render_ref(&self, area: Rect, buffer: &mut Buffer) {
        let logs = get_logs().unwrap();

        let log_lines: Vec<u16> = logs
            .iter()
            .map(|(kind, log)| match kind {
                LogKind::Debug => log.lines().count() as u16 + 1,
                _ => log.lines().count() as u16,
            })
            .collect();

        let logs = logs.iter().map(|(kind, log)| {
            let (prefix, style) = match kind {
                LogKind::Info => ("[info]:", Style::default().fg(Color::Blue)),
                LogKind::Success => ("[success]:", Style::default().fg(Color::Green)),
                LogKind::Error => ("[error]:", Style::default().fg(Color::Red)),
                LogKind::Warn => ("[warn]:", Style::default().fg(Color::Yellow)),
                LogKind::Debug => ("[debug]:", Style::default().fg(Color::White)),
            };
            (format!("{} {}", prefix, log), style)
        });

        let temp = widget!({

            Layout(
                constraints: [Constraint::Fill(1), Constraint::Fill(1)],
                direction: Direction::Horizontal
            ) {
                For (
                    (log, style) in logs.clone(),
                    constraints: Constraint::from_lengths(log_lines.clone()),
                    direction: Direction::Vertical
                ) {
                    Text::raw(log, style: style)
                },
                For (
                    (log, style) in logs.clone(),
                    constraints: Constraint::from_lengths(log_lines.clone()),
                    direction: Direction::Vertical
                ) {
                    Text::raw(log, style: style)
                }
            }

        });

        ui!((area, buffer) => {
            {{ temp }}
        });
    }
}
