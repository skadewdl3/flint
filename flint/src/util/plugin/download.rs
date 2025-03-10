use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::util::plugin;
use crate::util::toml::Config;
use crate::widgets::logs::{LogKind, add_log};

use super::PluginKind;

pub fn clone_plugin_folders(
    repo_url: &str,
    plugin_kind: PluginKind,
    plugin_ids: Vec<&String>,
) -> Result<PathBuf, Box<dyn Error>> {
    add_log(
        LogKind::Info,
        format!(
            "Starting plugin clone process for {} plugins",
            plugin_kind.to_string()
        ),
    );

    // Determine final destination path
    let final_dest_path = plugin::dir();

    add_log(
        LogKind::Info,
        format!("Downloading plugins to: {}", final_dest_path.display()),
    );

    // Create a temporary directory for git operations
    let temp_path = std::env::temp_dir().join("flint-plugins-temp");
    add_log(
        LogKind::Info,
        format!("Setting up temporary directory at: {}", temp_path.display()),
    );

    // Create temporary directory
    fs::create_dir_all(&temp_path)?;

    add_log(
        LogKind::Info,
        format!("Created temporary directory: {}", temp_path.display()),
    );

    // Create the kind subfolder
    let kind_str = &plugin_kind.to_string();
    let kind_path = final_dest_path.join(kind_str);
    add_log(
        LogKind::Info,
        format!("Plugin type directory: {}", kind_path.display()),
    );

    // Make sure the final plugin type directory exists
    fs::create_dir_all(&kind_path)?;
    add_log(
        LogKind::Info,
        format!("Created plugin type directory: {}", kind_path.display()),
    );

    // Check if git is installed and available
    add_log(LogKind::Info, "Checking if git is installed".to_string());
    let git_check = Command::new("git").arg("--help").output();

    match git_check {
        Ok(output) => {
            if !output.status.success() {
                add_log(
                    LogKind::Error,
                    "Git command failed. Is git installed correctly?".to_string(),
                );
                return Err("Git is not available or not working properly".into());
            }
            add_log(LogKind::Info, "Git is available and working".to_string());
        }
        Err(_) => {
            add_log(
                LogKind::Error,
                "Could not execute git command. Is git installed?".to_string(),
            );
            return Err("Git is not installed or not in PATH".into());
        }
    }

    add_log(
        LogKind::Info,
        format!("Cloning repository metadata from {}", repo_url),
    );

    // Do git operations in the temporary path
    let mut cmd = Command::new("git");
    let cmd = cmd
        .arg("clone")
        .arg("--filter=blob:none")
        .arg("--sparse")
        .arg(repo_url)
        .arg(&temp_path);

    add_log(
        LogKind::Info,
        format!("Executing git clone with sparse checkout filter"),
    );

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
        add_log(
            LogKind::Info,
            format!(
                "Cleaning up temporary directory after failed clone: {}",
                temp_path.display()
            ),
        );
        let _ = fs::remove_dir_all(&temp_path);
        return Err("Failed to clone repository".into());
    }

    add_log(
        LogKind::Info,
        "Repository cloned successfully, setting up sparse checkout".to_string(),
    );

    let mut sparse_paths = Vec::new();
    for id in &plugin_ids {
        sparse_paths.push(format!("flint-plugins/{}/{}", kind_str, id));
    }

    add_log(
        LogKind::Info,
        format!(
            "Setting sparse checkout for {} plugin paths",
            sparse_paths.len()
        ),
    );

    let output = Command::new("git")
        .current_dir(&temp_path)
        .arg("sparse-checkout")
        .arg("set")
        .args(&sparse_paths)
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
        add_log(
            LogKind::Info,
            format!(
                "Cleaning up after failed sparse-checkout: {}",
                temp_path.display()
            ),
        );
        let _ = fs::remove_dir_all(&temp_path);
        return Err("Failed to set sparse-checkout".into());
    }

    // Check if the plugins directory exists
    let temp_plugins_dir = temp_path.join("flint-plugins").join(kind_str);
    add_log(
        LogKind::Info,
        format!(
            "Checking for plugins directory: {}",
            temp_plugins_dir.display()
        ),
    );

    if !temp_plugins_dir.exists() {
        add_log(
            LogKind::Error,
            format!(
                "Plugins directory not found: {}",
                temp_plugins_dir.display()
            ),
        );
        let _ = fs::remove_dir_all(&temp_path);
        return Err(format!(
            "Plugins directory not found: {}",
            temp_plugins_dir.display()
        )
        .into());
    }

    add_log(
        LogKind::Info,
        format!(
            "Found plugins directory, copying {} plugins",
            plugin_ids.len()
        ),
    );

    // Copy each requested plugin to the final destination
    for id in &plugin_ids {
        let src_plugin_path = temp_plugins_dir.join(id);
        let dest_plugin_path = kind_path.join(id);

        add_log(LogKind::Info, format!("Processing plugin: {}", id));

        if src_plugin_path.exists() {
            // Remove the destination plugin if it already exists
            if dest_plugin_path.exists() {
                add_log(
                    LogKind::Info,
                    format!("Removing existing plugin: {}", dest_plugin_path.display()),
                );
                fs::remove_dir_all(&dest_plugin_path)?;
                add_log(
                    LogKind::Info,
                    format!("Removed existing plugin: {}", dest_plugin_path.display()),
                );
            }

            // Copy the plugin directory recursively
            add_log(
                LogKind::Info,
                format!(
                    "Copying plugin from {} to {}",
                    src_plugin_path.display(),
                    dest_plugin_path.display()
                ),
            );
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
    add_log(
        LogKind::Info,
        format!("Cleaning up temporary directory: {}", temp_path.display()),
    );
    fs::remove_dir_all(&temp_path)?;
    add_log(
        LogKind::Info,
        format!("Removed temporary directory: {}", temp_path.display()),
    );

    add_log(
        LogKind::Info,
        format!(
            "Successfully extracted plugin folders to {}",
            kind_path.display()
        ),
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
    add_log(
        LogKind::Info,
        format!(
            "Starting download of {} plugins for {}",
            ids.len(),
            kind.to_string()
        ),
    );
    let repo_url = "https://github.com/skadewdl3/flint";
    add_log(LogKind::Info, format!("Using repository URL: {}", repo_url));
    clone_plugin_folders(repo_url, kind.clone(), ids)?;
    add_log(
        LogKind::Info,
        format!("Completed downloading {} plugins", kind.to_string()),
    );
    Ok(())
}

pub fn download_plugins_from_config(toml: &Config) -> Result<(), Box<dyn Error>> {
    add_log(
        LogKind::Info,
        "Loading configuration from flint.toml".to_string(),
    );

    let linter_ids: Vec<&String> = toml.rules.keys().collect();
    let tester_ids: Vec<&String> = toml.tests.keys().collect();
    let ci_ids: Vec<&String> = toml.ci.keys().collect();
    let report_ids: Vec<&String> = toml.report.keys().collect();

    add_log(
        LogKind::Info,
        format!("Found {} test plugins in configuration", tester_ids.len()),
    );
    add_log(
        LogKind::Info,
        format!("Found {} lint plugins in configuration", linter_ids.len()),
    );
    add_log(
        LogKind::Info,
        format!("Found {} CI plugins in configuration", ci_ids.len()),
    );
    add_log(
        LogKind::Info,
        format!("Found {} report plugins in configuration", report_ids.len()),
    );

    add_log(
        LogKind::Info,
        "Starting download of all configured plugins".to_string(),
    );
    download_plugins(PluginKind::Test, tester_ids)?;
    download_plugins(PluginKind::Lint, linter_ids)?;
    download_plugins(PluginKind::Ci, ci_ids)?;
    download_plugins(PluginKind::Report, report_ids)?;
    add_log(
        LogKind::Info,
        "All plugins downloaded successfully".to_string(),
    );

    Ok(())
}
