use super::{helpers::add_helper_globals, Plugin};
use crate::app::AppResult;
use mlua::{Error, Function, Lua, LuaSerdeExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dependency {
    pub name: String,
    pub version: String,
}

pub fn get_dependencies(plugin: &Plugin) -> AppResult<HashMap<String, Vec<Dependency>>> {
    let lua = Lua::new();
    add_helper_globals(&lua);

    let deps_func: Result<Function, Error> = {
        let contents = std::fs::read_to_string(plugin.path.join("details.lua"))?;

        lua.load(contents)
            .exec()
            .map(|_| lua.globals().get("Dependencies").unwrap())
    };

    if let Ok(func) = deps_func {
        let deps_value = func.call::<mlua::Value>(()).unwrap();
        let deps: HashMap<String, Vec<Dependency>> = lua.from_value(deps_value).unwrap();
        Ok(deps)
    } else {
        // Return empty deps if no dependencies.lua exists
        Ok(HashMap::new())
    }
}

pub fn collect_dependencies(plugins: &Vec<Plugin>) -> AppResult<HashMap<String, Vec<Dependency>>> {
    let mut all_deps: HashMap<String, Vec<Dependency>> = HashMap::new();

    for plugin in plugins {
        if let Ok(deps) = plugin.get_dependencies() {
            for (manager, deps_list) in deps {
                all_deps
                    .entry(manager)
                    .or_insert_with(Vec::new)
                    .extend(deps_list);
            }
        }
    }

    // Deduplicate and resolve version conflicts
    deduplicate_dependencies(&mut all_deps);

    Ok(all_deps)
}

// TODO: Identify incompatible dependencies and perform resolution if possible.
fn deduplicate_dependencies(all_deps: &mut HashMap<String, Vec<Dependency>>) {
    for (_, deps) in all_deps.iter_mut() {
        // Use a map to track the latest version of each package
        let mut unique_deps: HashMap<String, Dependency> = HashMap::new();

        for dep in deps.iter() {
            if let Some(existing) = unique_deps.get(&dep.name) {
                // Handle "latest" version specifier
                if dep.version == "latest" {
                    unique_deps.insert(dep.name.clone(), dep.clone());
                    continue;
                }
                if existing.version == "latest" {
                    // Keep "latest" if it's already there
                    continue;
                }

                // If we've already seen this dependency, use semantic versioning to determine
                // which version to keep (preferring higher versions)
                if should_replace_version(&existing.version, &dep.version) {
                    unique_deps.insert(dep.name.clone(), dep.clone());
                }
            } else {
                unique_deps.insert(dep.name.clone(), dep.clone());
            }
        }

        // Replace the list with our deduplicated version
        *deps = unique_deps.into_values().collect();

        // Sort dependencies by name for consistency
        deps.sort_by(|a, b| a.name.cmp(&b.name));
    }
}

/// Determines if the new version should replace the existing version
/// Based on semantic versioning principles
fn should_replace_version(existing: &str, new: &str) -> bool {
    // Handle "latest" version specifier
    if new == "latest" {
        return true;
    }
    if existing == "latest" {
        return false;
    }

    // Parse the versions, stripping any semver operators
    let parse_version = |v: &str| -> Option<semver::Version> {
        let version_str = v.trim_start_matches(|c| !char::is_digit(c, 10));
        match semver::Version::parse(version_str) {
            Ok(v) => Some(v),
            Err(e) => None,
        }
    };

    if let (Some(existing_ver), Some(new_ver)) = (parse_version(existing), parse_version(new)) {
        // Return true if the new version is greater than the existing version
        return new_ver > existing_ver;
    }

    // Handle special cases for version ranges
    // If existing is a range and new is specific, prefer specific
    if existing.contains('^') || existing.contains('~') || existing.contains('*') {
        if !new.contains('^') && !new.contains('~') && !new.contains('*') {
            return true;
        }
    }

    // If both are ranges, prefer the one with the highest minimum
    if let (Some(existing_min), Some(new_min)) = (
        extract_minimum_version(existing),
        extract_minimum_version(new),
    ) {
        let result = new_min > existing_min;
        return result;
    }

    // Default to keeping the existing version
    false
}

/// Extracts the minimum version from a semver range
fn extract_minimum_version(version_range: &str) -> Option<semver::Version> {
    // Handle "latest" version specifier
    if version_range == "latest" {
        // Return a high version to ensure it's preferred
        return Some(semver::Version::new(9999, 0, 0));
    }

    // Handle caret ranges (^1.2.3)
    if version_range.starts_with('^') {
        let ver_str = &version_range[1..];
        return match semver::Version::parse(ver_str) {
            Ok(v) => Some(v),
            Err(e) => None,
        };
    }

    // Handle tilde ranges (~1.2.3)
    if version_range.starts_with('~') {
        let ver_str = &version_range[1..];
        return match semver::Version::parse(ver_str) {
            Ok(v) => Some(v),
            Err(e) => None,
        };
    }

    // Handle star ranges (1.2.*)
    if version_range.contains('*') {
        let parts: Vec<&str> = version_range.split('*').collect();
        let base_version = parts[0].trim_end_matches('.');

        // Try to parse as is, otherwise add zeros for missing components
        if let Ok(ver) = semver::Version::parse(base_version) {
            return Some(ver);
        } else {
            // Count the dots to determine components
            let dots = base_version.chars().filter(|&c| c == '.').count();
            let version_str = match dots {
                0 => format!("{}.0.0", base_version),
                1 => format!("{}.0", base_version),
                _ => base_version.to_string(),
            };
            return match semver::Version::parse(&version_str) {
                Ok(v) => Some(v),
                Err(e) => None,
            };
        }
    }

    // Handle range with comparison operators
    if version_range.starts_with('>') {
        let ver_str = version_range.trim_start_matches(|c| !char::is_digit(c, 10));
        return match semver::Version::parse(ver_str) {
            Ok(v) => Some(v),
            Err(e) => None,
        };
    }

    // Default fallback - try to parse as is
    match semver::Version::parse(version_range) {
        Ok(v) => Some(v),
        Err(e) => None,
    }
}
