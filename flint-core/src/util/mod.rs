use crossterm::event::{Event, KeyCode, KeyEvent};
use plugin::Plugin;
use std::{
    collections::{BTreeSet, HashMap},
    sync::OnceLock,
};

pub mod lang;
pub mod plugin;
pub mod toml;

pub static LANGUAGE_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();
pub static PLUGINS: OnceLock<BTreeSet<Plugin>> = OnceLock::new();
pub static PLUGIN_MAP: OnceLock<HashMap<String, BTreeSet<Plugin>>> = OnceLock::new();

pub use lang::{detect_languages, get_language_map};
pub use plugin::get_plugin_map;

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
