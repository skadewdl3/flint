use std::path::Path;

use crossterm::event::{Event, KeyCode, KeyEvent};

pub mod lang;
pub mod toml;

use flint_utils::{set_flag, Result};
pub use lang::{detect_languages, get_language_map};

use crate::app::AppArgs;

pub fn handle_key_events(
    event: Event,
    callback: impl FnOnce(KeyEvent, KeyCode) -> Result<()>,
) -> Result<()> {
    if let Event::Key(key_event) = event {
        return callback(key_event, key_event.code);
    }
    Ok(())
}

pub fn handle_mouse_event(
    event: Event,
    callback: impl FnOnce(crossterm::event::MouseEventKind) -> Result<()>,
) -> Result<()> {
    if let Event::Mouse(mouse_event) = event {
        return callback(mouse_event.kind);
    }
    Ok(())
}

pub fn handle_global_flags(app_args: &AppArgs) {
    if let Some(ref plugins_dir) = app_args.plugins_dir {
        let path = Path::new(plugins_dir);
        let plugins_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().unwrap().join(&path)
        };
        set_flag!(plugins_dir, plugins_path);
    }

    if let Some(ref config_path) = app_args.config_path {
        let path = Path::new(config_path);
        let config_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().unwrap().join(&path)
        };
        // Update the current_dir based on the config path's parent directory
        let current_dir = config_path.parent().unwrap_or(Path::new("")).to_path_buf();
        set_flag!(current_dir, current_dir);
        set_flag!(config_path, config_path);
    }

    set_flag!(no_install, app_args.no_install);
}
