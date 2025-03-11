use std::{
    path::{Path, PathBuf},
    sync::{LazyLock, RwLock},
};

use crate::app::AppArgs;

pub struct Flags {
    pub non_interactive: bool,
    pub plugins_dir: PathBuf,
    pub config_path: PathBuf,
    pub current_dir: PathBuf,
    pub no_install: bool,
}

// Create a static global instance with RwLock
pub static GLOBAL_FLAGS: LazyLock<RwLock<Flags>> = LazyLock::new(|| {
    RwLock::new(Flags {
        non_interactive: false,
        plugins_dir: crate::plugin::dir(),
        config_path: std::env::current_dir().unwrap().join("flint.toml"),
        current_dir: std::env::current_dir().unwrap(),
        no_install: false,
    })
});

#[macro_export]
macro_rules! get_flag {
    ($name:ident) => {{
        use $crate::util::flags::GLOBAL_FLAGS;

        // Access the flag value through the RwLock read guard
        &GLOBAL_FLAGS.read().unwrap().$name.clone()
    }};
}

#[macro_export]
macro_rules! set_flag {
    ($name:ident, $value:expr) => {{
        use $crate::util::flags::GLOBAL_FLAGS;

        // Acquire a write lock to safely modify the flag
        let mut flags = GLOBAL_FLAGS.write().unwrap();
        flags.$name = $value;
    }};
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
