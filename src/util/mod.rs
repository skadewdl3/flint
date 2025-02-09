use crossterm::event::{Event, KeyCode, KeyEvent};

use crate::widgets::AppStatus;

pub mod file;
pub mod plugin;
pub mod toml;

pub fn handle_key_events<'a>(
    event: Event,
    callback: impl FnOnce(KeyEvent, KeyCode) -> AppStatus<'a>,
) -> AppStatus<'a> {
    if let Event::Key(key_event) = event {
        return callback(key_event, key_event.code);
    }
    AppStatus::Ok
}
