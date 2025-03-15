use std::{
    path::PathBuf,
    sync::{LazyLock, RwLock},
};

use directories::ProjectDirs;

pub fn plugin_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        return PathBuf::from("./flint-plugins");
    } else if let Some(proj_dirs) = ProjectDirs::from("com", "Flint", "flint") {
        let plugins_path = proj_dirs.data_dir().to_path_buf().join("plugins");
        if !plugins_path.exists() {
            std::fs::create_dir_all(&plugins_path).expect("Failed to create plugins directory");
        }
        plugins_path
    } else {
        panic!("Unable to determine project directories");
    }
}

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
        plugins_dir: plugin_dir(),
        config_path: std::env::current_dir().unwrap().join("flint.toml"),
        current_dir: std::env::current_dir().unwrap(),
        no_install: false,
    })
});

#[macro_export]
macro_rules! get_flag {
    ($name:ident) => {{
        use $crate::flags::GLOBAL_FLAGS;

        // Access the flag value through the RwLock read guard
        GLOBAL_FLAGS.read().unwrap().$name.clone()
    }};
}

#[macro_export]
macro_rules! set_flag {
    ($name:ident, $value:expr) => {{
        use $crate::flags::GLOBAL_FLAGS;

        // Acquire a write lock to safely modify the flag
        let mut flags = GLOBAL_FLAGS.write().unwrap();
        flags.$name = $value;
    }};
}
