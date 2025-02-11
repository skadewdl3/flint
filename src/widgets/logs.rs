use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard};

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
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
}

pub static LOGS: OnceLock<RwLock<Vec<(LogKind, String)>>> = OnceLock::new();

#[derive(Debug, Default)]
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

impl AppWidget for LogsWidget {
    fn draw(&mut self, frame: &mut Frame) -> AppStatus {
        if let Ok(logs) = get_logs() {
            let length = logs.len();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(Constraint::from_lengths(vec![1; length]))
                .split(frame.area());

            for i in 0..logs.len() {
                let (kind, log) = logs.get(i).unwrap();
                let prefix = match kind {
                    LogKind::Info => "[info]:",
                    LogKind::Success => "[success]:",
                    LogKind::Error => "[error]:",
                    LogKind::Warn => "[warn]:",
                };

                let message = format!("{} {}", prefix, log);

                let style = match kind {
                    LogKind::Info => Style::default().fg(Color::Blue),
                    LogKind::Success => Style::default().fg(Color::Green),
                    LogKind::Error => Style::default().fg(Color::Red),
                    LogKind::Warn => Style::default().fg(Color::Yellow),
                };

                let text = Text::styled(message, style);
                frame.render_widget(text, layout[i]);
            }
        }
        AppStatus::Ok
    }
}
