use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::util::plugin;
use crate::util::toml::Config;
use crate::widgets::logs::{add_log, LogKind};

use super::PluginKind;

pub fn clone_plugin_folders(
    repo_url: &str,
    plugin_kind: PluginKind,
    plugin_ids: Vec<&String>,
    destination: Option<PathBuf>,
) -> Result<PathBuf, Box<dyn Error>> {
    // Determine final destination path
    let final_dest_path = destination.unwrap_or_else(|| {
        if cfg!(debug_assertions) {
            Path::new("./downloaded-plugins").to_path_buf()
        } else {
            // Replace with your actual plugin directory function
            plugin::dir()
        }
    });

    // Create a temporary directory for git operations
    let temp_path = std::env::temp_dir().join("flint-plugins-temp");

    // Create temporary directory
    fs::create_dir_all(&temp_path)?;

    add_log(
        LogKind::Info,
        format!("Created temporary directory: {}", temp_path.display()),
    );

    // Create the kind subfolder
    let kind_str = &plugin_kind.to_string();
    let kind_path = final_dest_path.join(kind_str);

    // Make sure the final plugin type directory exists
    fs::create_dir_all(&kind_path)?;

    // Do git operations in the temporary path
    let mut cmd = Command::new("git");
    let cmd = cmd
        .arg("clone")
        .arg("--filter=blob:none")
        .arg("--sparse")
        .arg(repo_url)
        .arg(&temp_path);

    let output = cmd.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let status = output.status;

    if !stdout.is_empty() {
        add_log(LogKind::Info, format!("[git clone]: {}", stdout));
    }

    if !stderr.is_empty() {
        add_log(LogKind::Error, format!("[git clone]: {}", stderr));
    }

    if !status.success() {
        // Clean up the temporary directory if clone fails
        let _ = fs::remove_dir_all(&temp_path);
        return Err("Failed to clone repository".into());
    }

    let mut sparse_paths = Vec::new();
    for id in &plugin_ids {
        sparse_paths.push(format!("flint-plugins/{}/{}", kind_str, id));
    }

    let output = Command::new("git")
        .current_dir(&temp_path)
        .arg("sparse-checkout")
        .arg("set")
        .args(sparse_paths)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let status = output.status;

    if !stdout.is_empty() {
        add_log(LogKind::Info, format!("[git sparse-checkout]: {}", stdout));
    }

    if !stderr.is_empty() {
        add_log(LogKind::Error, format!("[git sparse-checkout]: {}", stderr));
    }

    if !status.success() {
        // Clean up the temporary directory if sparse-checkout fails
        let _ = fs::remove_dir_all(&temp_path);
        return Err("Failed to set sparse-checkout".into());
    }

    // Check if the plugins directory exists
    let temp_plugins_dir = temp_path.join("flint-plugins").join(kind_str);
    if !temp_plugins_dir.exists() {
        let _ = fs::remove_dir_all(&temp_path);
        return Err(format!(
            "Plugins directory not found: {}",
            temp_plugins_dir.display()
        )
        .into());
    }

    // Copy each requested plugin to the final destination
    for id in &plugin_ids {
        let src_plugin_path = temp_plugins_dir.join(id);
        let dest_plugin_path = kind_path.join(id);

        if src_plugin_path.exists() {
            // Remove the destination plugin if it already exists
            if dest_plugin_path.exists() {
                fs::remove_dir_all(&dest_plugin_path)?;
                add_log(
                    LogKind::Info,
                    format!("Removed existing plugin: {}", dest_plugin_path.display()),
                );
            }

            // Copy the plugin directory recursively
            copy_dir_all(&src_plugin_path, &dest_plugin_path)?;
            add_log(
                LogKind::Info,
                format!(
                    "Copied plugin from {} to {}",
                    src_plugin_path.display(),
                    dest_plugin_path.display()
                ),
            );
        } else {
            add_log(
                LogKind::Warn,
                format!("Plugin not found: {}", src_plugin_path.display()),
            );
        }
    }

    // Clean up the temporary directory
    fs::remove_dir_all(&temp_path)?;
    add_log(
        LogKind::Info,
        format!("Removed temporary directory: {}", temp_path.display()),
    );

    println!(
        "Successfully extracted plugin folders to {}",
        kind_path.display()
    );

    Ok(kind_path)
}

// Helper function to recursively copy directories
fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), Box<dyn Error>> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let path = entry.path();

        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path)?;
        }
    }

    Ok(())
}

// Example usage
pub fn download_plugins(kind: PluginKind, ids: Vec<&String>) -> Result<(), Box<dyn Error>> {
    let repo_url = "https://github.com/skadewdl3/flint";
    clone_plugin_folders(repo_url, kind, ids, None)?;
    Ok(())
}

pub fn download_plugins_from_config(config: &Config) -> Result<(), Box<dyn Error>> {
    let toml = Config::load(std::env::current_dir().unwrap().join("flint.toml")).unwrap();

    let linter_ids: Vec<&String> = toml.rules.keys().collect();
    let tester_ids: Vec<&String> = toml.tests.keys().collect();

    download_plugins(PluginKind::Test, tester_ids)?;
    download_plugins(PluginKind::Lint, linter_ids)?;
    Ok(())
}
