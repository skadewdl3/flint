use std::path::PathBuf;

use crate::app_err;

pub fn load_from_file(file: &PathBuf) -> crate::Result<()> {
    let _ = dotenvy::from_filename(file)?;
    Ok(())
}

pub fn get_env_var(name: &str) -> crate::Result<String> {
    if let Ok(val) = dotenvy::var(name) {
        Ok(val)
    } else if let Ok(val) = std::env::var(name) {
        Ok(val)
    } else {
        app_err!("Could not find environment variable with the name {}", name)
    }
}
