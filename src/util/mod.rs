use crossterm::event::{Event, KeyCode, KeyEvent};

pub mod file;

pub mod toml;

pub fn handle_key_events(event: &Event, callback: impl FnOnce(KeyEvent, KeyCode)) {
    if let Event::Key(key_event) = event {
        callback(*key_event, key_event.code);
    }
}
