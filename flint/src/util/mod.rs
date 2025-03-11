use crossterm::event::{Event, KeyCode, KeyEvent};

pub mod flags;
pub mod lang;
pub mod logs;
pub mod toml;

pub use lang::{detect_languages, get_language_map};

use crate::app::AppResult;

pub fn handle_key_events(
    event: Event,
    callback: impl FnOnce(KeyEvent, KeyCode) -> AppResult<()>,
) -> AppResult<()> {
    if let Event::Key(key_event) = event {
        return callback(key_event, key_event.code);
    }
    Ok(())
}

pub fn handle_mouse_event(
    event: Event,
    callback: impl FnOnce(crossterm::event::MouseEventKind) -> AppResult<()>,
) -> AppResult<()> {
    if let Event::Mouse(mouse_event) = event {
        return callback(mouse_event.kind);
    }
    Ok(())
}
