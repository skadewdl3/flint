use crate::widgets::AppStatus;
use crossterm::event::{Event, KeyCode, KeyEvent};
use plugin::Plugin;
use std::{
    collections::{BTreeSet, HashMap},
    sync::OnceLock,
};

pub mod lang;
pub mod plugin;
pub mod thread_manager;
pub mod toml;

pub static LANGUAGE_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();
pub static PLUGINS: OnceLock<BTreeSet<Plugin>> = OnceLock::new();
pub static PLUGIN_MAP: OnceLock<HashMap<String, BTreeSet<Plugin>>> = OnceLock::new();

pub use lang::{detect_languages, get_language_map};
pub use plugin::{get_plugin_map, get_plugins_dir, list_plugins};

pub fn handle_key_events<'a>(
    event: Event,
    callback: impl FnOnce(KeyEvent, KeyCode) -> AppStatus<'a>,
) -> AppStatus<'a> {
    if let Event::Key(key_event) = event {
        return callback(key_event, key_event.code);
    }
    AppStatus::Ok
}
