use std::path::PathBuf;

pub fn load_from_file(file: &PathBuf) -> crate::Result<()> {
    let _ = dotenvy::from_filename(file)?;
    Ok(())
}

pub fn get_env_var(name: &str) -> crate::Result<String> {
    dotenvy::var(name).map_err(|e| e.into())
}
