use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::{error::Error, path::PathBuf};

use crate::util::plugin;
use crate::widgets::logs::{add_log, LogKind};

use super::{Plugin, PluginKind};

#[derive(Debug, Serialize, Deserialize)]
struct GitHubItem {
    name: String,
    path: String,
    #[serde(rename = "type")]
    item_type: String,
    download_url: Option<String>,
    url: String,
}

pub fn download_plugin(kind: PluginKind, id: &str) -> Result<(), Box<dyn Error>> {
    let folder_path = format!(
        "flint-plugins/{}/{}",
        match kind {
            PluginKind::Lint => "lint",
            PluginKind::Test => "test",
        },
        id
    );
    add_log(LogKind::Info, format!("Trying to download {}", folder_path));
    let local_path = if cfg!(debug_assertions) {
        Path::new("./downloaded-plugins").to_path_buf()
    } else {
        plugin::dir()
    };
    let local_path = local_path.to_str().unwrap();

    // Set up the HTTP client with headers
    let client = setup_client()?;

    // Create the local directory if it doesn't exist
    fs::create_dir_all(local_path)?;

    // First, get the folder contents
    let contents_url = format!(
        "https://api.github.com/repos/skadewdl3/flint/contents/{}",
        folder_path
    );

    println!("Fetching folder structure from: {}", contents_url);

    let response = client.get(&contents_url).send()?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch folder contents: {} - {}",
            response.status(),
            response.text()?
        )
        .into());
    }

    let items: Vec<GitHubItem> = response.json()?;

    // Download each item
    for item in items {
        let target_path = format!("{}/{}", local_path, item.name);

        if item.item_type == "dir" {
            // Handle subdirectories - recursive call
            download_folder(&client, &item.url, &target_path)?;
        } else if item.item_type == "file" {
            // Download the file
            if let Some(download_url) = item.download_url {
                download_file(&client, &download_url, &target_path)?;
            } else {
                println!("Warning: No download URL for file: {}", item.path);
            }
        }
    }

    println!("Download completed successfully to {}", local_path);
    Ok(())
}

fn setup_client() -> Result<Client, Box<dyn Error>> {
    // Optional: GitHub token for higher rate limits
    let github_token = option_env!("GITHUB_TOKEN");

    // Set up the HTTP client
    let builder = reqwest::blocking::ClientBuilder::new();
    let mut headers = reqwest::header::HeaderMap::new();

    // Add GitHub token if available
    if let Some(token) = github_token {
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("token {}", token))?,
        );
    }
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("rust-github-folder-downloader"),
    );

    let client = builder.default_headers(headers).build().unwrap();
    Ok(client)
}

fn download_folder(client: &Client, url: &str, local_path: &str) -> Result<(), Box<dyn Error>> {
    // Create the local directory for the subfolder
    fs::create_dir_all(local_path)?;

    println!("Downloading folder to: {}", local_path);

    // Get the folder contents
    let response = client.get(url).send()?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch subfolder contents: {} - {}",
            response.status(),
            response.text()?
        )
        .into());
    }

    let items: Vec<GitHubItem> = response.json()?;

    // Download each item
    for item in items {
        let target_path = format!("{}/{}", local_path, item.name);

        if item.item_type == "dir" {
            // Recursive call for subdirectories
            download_folder(client, &item.url, &target_path)?;
        } else if item.item_type == "file" {
            // Download the file
            if let Some(download_url) = item.download_url {
                download_file(client, &download_url, &target_path)?;
            } else {
                println!("Warning: No download URL for file: {}", item.path);
            }
        }
    }

    Ok(())
}

fn download_file(client: &Client, url: &str, local_path: &str) -> Result<(), Box<dyn Error>> {
    println!("Downloading file to: {}", local_path);

    // Get the file content
    let response = client.get(url).send()?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download file: {} - {}",
            response.status(),
            response.text()?
        )
        .into());
    }

    // Create parent directories if they don't exist
    if let Some(parent) = Path::new(local_path).parent() {
        fs::create_dir_all(parent)?;
    }

    // Write to file
    let content = response.bytes()?;
    let mut file = fs::File::create(local_path)?;
    file.write_all(&content)?;

    Ok(())
}
