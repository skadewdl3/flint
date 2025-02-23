use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use flint_macros::{ui, widget};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Paragraph, Widget, Wrap},
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

pub static LOGS: RwLock<Vec<(LogKind, String)>> = RwLock::new(vec![]);

#[derive(Debug, Default, Copy, Clone)]
pub struct LogsWidget;

pub fn get_logs() -> Result<
    RwLockReadGuard<'static, Vec<(LogKind, String)>>,
    std::sync::PoisonError<RwLockReadGuard<'static, Vec<(LogKind, String)>>>,
> {
    LOGS.read()
}

pub fn get_logs_mut() -> Result<
    RwLockWriteGuard<'static, Vec<(LogKind, String)>>,
    std::sync::PoisonError<RwLockWriteGuard<'static, Vec<(LogKind, String)>>>,
> {
    LOGS.write()
}

pub fn add_log(kind: LogKind, message: String) {
    use std::fs::OpenOptions;
    use std::io::Write;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs.txt")
        .unwrap();

    let prefix = match kind {
        LogKind::Info => "[info]:",
        LogKind::Success => "[success]:",
        LogKind::Error => "[error]:",
        LogKind::Warn => "[warn]:",
        LogKind::Debug => "[debug]:",
    };

    let log = format!("{} {}", prefix, message);
    writeln!(file, "{}", log).unwrap();
    get_logs_mut().unwrap().push((kind, log));
}

fn get_style(kind: &LogKind) -> Style {
    match kind {
        LogKind::Info => Style::default().fg(Color::Blue),
        LogKind::Success => Style::default().fg(Color::Green),
        LogKind::Error => Style::default().fg(Color::Red),
        LogKind::Warn => Style::default().fg(Color::Yellow),
        LogKind::Debug => Style::default().fg(Color::White),
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

        ui!((area, buffer) => {
            For (
                (kind, log) in logs.iter(),
                constraints: Constraint::from_lengths(log_lines),
                direction: Direction::Vertical
            ) {
                Paragraph::new(
                    log.as_str(),
                    style: get_style(kind),
                    wrap: Wrap { trim: true }
                )
            }
        });
    }
}
