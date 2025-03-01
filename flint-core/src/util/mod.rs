use crossterm::event::{Event, KeyCode, KeyEvent};

pub mod lang;
pub mod plugin;
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
