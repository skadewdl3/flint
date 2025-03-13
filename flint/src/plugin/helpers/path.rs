use std::path::Path;

use directories::UserDirs;
use mlua::{Lua, Table, Variadic};

use crate::{app::AppResult, get_flag};

pub fn path_helpers(lua: &Lua) -> AppResult<Table> {
    let path = lua.create_table()?;

    let cwd = lua.create_function(|lua, ()| {
        let cwd = get_flag!(current_dir);
        Ok(lua.create_string(cwd.to_string_lossy().as_ref())?)
    })?;

    let path_resolve = lua.create_function(|lua, paths: mlua::Variadic<String>| {
        use std::path::{Path, PathBuf};

        let cwd = get_flag!(current_dir);
        let mut result = PathBuf::new();
        let mut absolute = false;

        // Process each path segment similar to Node.js path.resolve
        for path in paths.iter() {
            let path_obj = Path::new(path);

            // If path is absolute, reset result and set absolute flag
            if path_obj.is_absolute() {
                result = PathBuf::from(path);
                absolute = true;
            } else if path.starts_with("~")
                && path.len() > 1
                && (path.len() == 1 || path.chars().nth(1) == Some('/'))
            {
                // Handle home directory with ~
                match UserDirs::new() {
                    Some(user_dirs) => {
                        let home = user_dirs.home_dir();
                        if path.len() > 1 {
                            result = home.join(&path[2..]);
                        } else {
                            result = home.to_path_buf();
                        }
                        absolute = true;
                    }
                    None => (),
                }
            } else {
                // For relative paths, append to result
                if !absolute {
                    // If this is the first path and it's relative, start from cwd
                    if result.as_os_str().is_empty() {
                        result = cwd.clone();
                    }
                }
                result = result.join(path);
            }
        }

        // If no paths provided, return cwd
        if result.as_os_str().is_empty() {
            result = cwd.clone();
        }

        // Normalize the path
        if let Ok(canonicalized) = result.canonicalize() {
            result = canonicalized;
        }

        Ok(lua.create_string(result.to_string_lossy().as_ref())?)
    })?;

    let path_join = lua.create_function(|lua, paths: mlua::Variadic<String>| {
        use std::path::Path;

        // If there are no path segments, return empty string
        if paths.len() == 0 {
            return Ok(lua.create_string("")?);
        }

        // Node.js path.join() just combines segments with the platform-specific separator
        // and normalizes the result, but it doesn't resolve to absolute paths
        let mut result = String::new();

        for (i, path) in paths.iter().enumerate() {
            // Skip empty segments (but preserve them at the beginning)
            if path.is_empty() && i > 0 {
                continue;
            }

            // Add separator between segments
            if i > 0 && !result.is_empty() && !result.ends_with(std::path::MAIN_SEPARATOR) {
                result.push(std::path::MAIN_SEPARATOR);
            }

            // Add the path segment
            result.push_str(path);
        }

        // Normalize the path (remove unnecessary separators/dots)
        let normalized = Path::new(&result).to_string_lossy();

        Ok(lua.create_string(normalized.as_ref())?)
    })?;

    let path_ls = lua.create_function(|lua, path: Option<String>| {
        use std::fs;

        // Get the directory to list
        let dir_path = match path {
            Some(p) => p,
            None => {
                // Default to current directory if no path provided
                let cwd = get_flag!(current_dir);
                cwd.to_string_lossy().to_string()
            }
        };

        // Read directory contents
        let entries = match fs::read_dir(&dir_path) {
            Ok(entries) => entries,
            Err(err) => {
                return Err(mlua::Error::runtime(format!(
                    "Failed to read directory: {}",
                    err
                )))
            }
        };

        // Create a Lua table to hold the results
        let results = lua.create_table()?;

        // Populate the table with file/directory names
        for (i, entry) in entries.enumerate() {
            match entry {
                Ok(entry) => {
                    let file_name = entry.file_name();
                    let name = file_name.to_string_lossy().to_string();
                    results.raw_set(i + 1, name)?;
                }
                Err(err) => {
                    return Err(mlua::Error::runtime(format!(
                        "Error reading entry: {}",
                        err
                    )))
                }
            }
        }

        Ok(results)
    })?;

    let path_relative = lua.create_function(|lua, args: Variadic<String>| {
        if args.len() != 2 {
            return Err(mlua::Error::RuntimeError(
                "Expected exactly two arguments: file_path and current_dir".into(),
            ));
        }

        let file_path = Path::new(&args[0]);
        let current_dir = Path::new(&args[1]);

        match file_path.strip_prefix(current_dir) {
            Ok(relative) => Ok(lua.create_string(relative.to_string_lossy().as_ref())?),
            Err(_) => Err(mlua::Error::RuntimeError(
                "Could not determine relative path".into(),
            )),
        }
    })?;

    path.set("join", path_join)?;
    path.set("resolve", path_resolve)?;
    path.set("ls", path_ls)?;
    path.set("cwd", cwd)?;
    path.set("relative", path_relative)?;

    Ok(path)
}
