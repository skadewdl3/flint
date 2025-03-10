use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::app::AppResult;
use crate::util::plugin;
use crate::util::toml::Config;
use crate::{app_err, error};
use crate::{cmd, info};
use crate::{success, warn};

use super::PluginKind;

pub fn clone_plugin_folders(
    repo_url: &str,
    plugin_kind: PluginKind,
    plugin_ids: Vec<&String>,
) -> AppResult<PathBuf> {
    info!(
        "Starting plugin clone process for {} plugins",
        plugin_kind.to_string()
    );

    // Determine final destination path
    let final_dest_path = plugin::dir();

    info!("Downloading plugins to: {}", final_dest_path.display());

    // Create a temporary directory for git operations
    // TODO: Use ProjDirs crate here instead of std::env::temp_dir()
    let temp_path = std::env::temp_dir().join("flint-plugins-temp");

    // Create temporary directory
    fs::create_dir_all(&temp_path)?;

    // Create the kind subfolder
    let kind_str = &plugin_kind.to_string();
    let kind_path = final_dest_path.join(kind_str);

    // Make sure the final plugin type directory exists
    fs::create_dir_all(&kind_path)?;

    // Check if git is installed and available
    info!("Checking if git is installed");
    let git_check = cmd!["git", "--help"].output().map_err(|e| {
        app_err!(
            "Git is not instaled on this system or not in PATH.\nError message: {}",
            e
        )
    })?;

    if !git_check.status.success() {
        return Err(app_err!("Git is not available or not working properly"));
    }

    info!("Cloning repository metadata from {}", repo_url);

    // Do git operations in the temporary path
    // Define a helper macro for creating commands

    let output = cmd![
        "git",
        "clone",
        "--filter=blob:none",
        "--sparse",
        repo_url,
        &temp_path
    ]
    .output()?;

    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let status = output.status;

    if !status.success() {
        info!(
            "Cleaning up temporary directory after failed clone: {}",
            temp_path.display()
        );
        _ = fs::remove_dir_all(&temp_path).map_err(|e| {
            app_err!("Failed to remove temporary directory: {}", e);
        });

        return Err(app_err!(
            "Failed to clone repository.\n Git Clone output: {}",
            stderr
        ));
    }

    if !status.success() {
        // Clean up the temporary directory if clone fails
        return Err(app_err!("Failed to clone repository"));
    }

    let mut sparse_paths = Vec::new();
    for id in &plugin_ids {
        sparse_paths.push(format!("flint-plugins/{}/{}", kind_str, id));
    }

    info!(
        "Metadata downloaded. Setting up sparse checkout for {} plugin paths",
        sparse_paths.len()
    );
    let output = cmd!["git", "sparse-checkout", "set"]
        .current_dir(&temp_path)
        .args(&sparse_paths)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let status = output.status;

    if !stdout.is_empty() {
        info!("[git sparse-checkout]: {}", stdout);
    }

    if !stderr.is_empty() {
        error!("[git sparse-checkout]: {}", stderr);
    }

    if !status.success() {
        // Clean up the temporary directory if sparse-checkout fails
        info!(
            "Cleaning up after failed sparse-checkout: {}",
            temp_path.display()
        );
        let _ = fs::remove_dir_all(&temp_path);
        return Err(app_err!("Failed to set sparse-checkout"));
    }

    // Check if the plugins directory exists
    let temp_plugins_dir = temp_path.join("flint-plugins").join(kind_str);
    info!(
        "Checking for plugins directory: {}",
        temp_plugins_dir.display()
    );

    if !temp_plugins_dir.exists() {
        error!(
            "Plugins directory not found: {}",
            temp_plugins_dir.display()
        );
        _ = fs::remove_dir_all(&temp_path).map_err(|e| {
            app_err!(
                "Failed to clean up temporary directory.\nError message: {}",
                e
            )
        });
        return Err(app_err!(
            "Plugins directory not found: {}",
            temp_plugins_dir.display()
        ));
    }

    // Copy each requested plugin to the final destination
    for id in &plugin_ids {
        let src_plugin_path = temp_plugins_dir.join(id);
        let dest_plugin_path = kind_path.join(id);

        if src_plugin_path.exists() {
            // Remove the destination plugin if it already exists
            if dest_plugin_path.exists() {
                fs::remove_dir_all(&dest_plugin_path)?;
            }

            // Copy the plugin directory recursively
            copy_dir_all(&src_plugin_path, &dest_plugin_path)?;
        } else {
            warn!("Plugin not found: {}", src_plugin_path.display());
        }
    }

    // Clean up the temporary directory
    info!("Cleaning up temporary directory: {}", temp_path.display());
    fs::remove_dir_all(&temp_path)?;

    success!("Successfully downloaded {} plugins", kind_path.display());

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
    info!(
        "Starting download of {} {} plugins",
        ids.len(),
        kind.to_string()
    );
    let repo_url = "https://github.com/skadewdl3/flint";
    info!("Source repository: {}", repo_url);
    clone_plugin_folders(repo_url, kind.clone(), ids)?;
    success!("Completed downloading {} plugins", kind.to_string());
    Ok(())
}

pub fn download_plugins_from_config(toml: &Config) -> Result<(), Box<dyn Error>> {
    info!("Loading configuration from flint.toml");

    let linter_ids: Vec<&String> = toml.rules.keys().collect();
    let tester_ids: Vec<&String> = toml.tests.keys().collect();
    let ci_ids: Vec<&String> = toml.ci.keys().collect();
    let report_ids: Vec<&String> = toml.report.keys().collect();

    info!("Found {} test plugins in configuration", tester_ids.len());
    info!("Found {} lint plugins in configuration", linter_ids.len());
    info!("Found {} CI plugins in configuration", ci_ids.len());
    info!("Found {} report plugins in configuration", report_ids.len());

    info!("Starting download of all configured plugins");
    if tester_ids.len() > 0 {
        download_plugins(PluginKind::Test, tester_ids)?;
    }
    if linter_ids.len() > 0 {
        download_plugins(PluginKind::Lint, linter_ids)?;
    }
    if ci_ids.len() > 0 {
        download_plugins(PluginKind::Ci, ci_ids)?;
    }
    if report_ids.len() > 0 {
        download_plugins(PluginKind::Report, report_ids)?;
    }
    success!("All plugins downloaded successfully");

    Ok(())
}
