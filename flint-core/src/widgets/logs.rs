use flint_macros::{ui, widget};
use ratatui::text::{Line, Text};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Padding, Paragraph, StatefulWidget, Widget},
};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

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

// Define a state to keep track of scrolling position
#[derive(Debug, Clone, Copy)]
pub struct LogsState {
    scroll: usize,
}

impl Default for LogsState {
    fn default() -> Self {
        Self { scroll: 0 }
    }
}

impl LogsState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_add(amount);
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_sub(amount);
    }

    pub fn reset_scroll(&mut self) {
        self.scroll = 0;
    }

    // Scroll to bottom of logs
    pub fn scroll_to_bottom(&mut self, total_lines: usize, visible_lines: usize) {
        if total_lines > visible_lines {
            self.scroll = total_lines - visible_lines;
        } else {
            self.scroll = 0;
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
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
    Style::default().fg(match kind {
        LogKind::Info => Color::Blue,
        LogKind::Success => Color::Green,
        LogKind::Error => Color::Red,
        LogKind::Warn => Color::Yellow,
        LogKind::Debug => Color::White,
    })
}

// Changed from Widget to StatefulWidget for scrolling functionality
impl StatefulWidget for LogsWidget {
    type State = LogsState;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        let logs = get_logs().unwrap();

        let all_log_lines = logs
            .iter()
            .flat_map(|(kind, log)| {
                log.split('\n')
                    .map(|line| Line::from(line.to_string()).style(get_style(kind)))
                    .collect::<Vec<Line>>()
            })
            .collect::<Vec<Line>>();

        let total_lines = all_log_lines.len();

        // Calculate max viewable lines in area (accounting for borders)
        let max_visible_lines = area.height.saturating_sub(2) as usize;

        // Ensure scroll doesn't go beyond possible range
        if total_lines > max_visible_lines && state.scroll > total_lines - max_visible_lines {
            state.scroll = total_lines - max_visible_lines;
        }

        // Select only the lines that should be visible based on scroll position
        let visible_log_lines = if !all_log_lines.is_empty() {
            let start = state.scroll.min(total_lines.saturating_sub(1));
            let end = (start + max_visible_lines).min(total_lines);
            all_log_lines[start..end].to_vec()
        } else {
            vec![]
        };

        let text = Text::from(visible_log_lines);
        let block = widget!({ Block::bordered(title: format!("Logs [{}-{}/{}]", state.scroll + 1, state.scroll + max_visible_lines.min(total_lines), total_lines), padding: Padding::horizontal(1)) });

        ui!((area, buffer) => {
           Paragraph::new(text, block: block)
        });
    }
}

// Keep the original Widget implementation for backward compatibility
impl Widget for LogsWidget {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let mut state = LogsState::default();
        StatefulWidget::render(self, area, buffer, &mut state);
    }
}
