use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use flint_macros::{ui, widget};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Padding, Paragraph, Widget, Wrap},
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

        let log_lines = logs
            .iter()
            .flat_map(|(kind, log)| {
                log.split('\n')
                    .map(|line| Line::from(line.to_string()).style(get_style(kind)))
                    .collect::<Vec<Line>>()
            })
            .collect::<Vec<Line>>();

        let text = Text::from(log_lines);

        let block = widget!({ Block::bordered(title: "Logs", padding: Padding::horizontal(1)) });

        ui!((area, buffer) => {
           Paragraph::new(text, block: block)
        });
    }
}
