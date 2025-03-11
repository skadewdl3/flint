use std::{
    collections::{BTreeSet, HashMap},
    path::Path,
    sync::OnceLock,
};

use ignore::Walk;

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Language {
    Supported(String),
    Unsupported(String),
}

pub static LANGUAGE_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();

pub fn get_language_map() -> &'static HashMap<String, String> {
    LANGUAGE_MAP.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert("rs".to_string(), "Rust".to_string());
        map.insert("py".to_string(), "Python".to_string());
        map.insert("js".to_string(), "JavaScript".to_string());
        map.insert("cpp".to_string(), "C++".to_string());
        map.insert("java".to_string(), "Java".to_string());
        map.insert("ts".to_string(), "TypeScript".to_string());
        map.insert("cs".to_string(), "C#".to_string());
        map.insert("go".to_string(), "Go".to_string());
        map.insert("swift".to_string(), "Swift".to_string());
        map.insert("kt".to_string(), "Kotlin".to_string());
        map
    })
}

pub fn detect_languages<'a>(project_path: impl Into<&'a str>) -> BTreeSet<Language> {
    let mut languages = BTreeSet::new();
    let path = Path::new(project_path.into());
    for result in Walk::new(path) {
        if let Ok(entry) = result {
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if let Some(ext) = extension.to_str() {
                        languages.insert(ext.to_string());
                    }
                }
            }
        }
    }

    let supported_languages: BTreeSet<String> = crate::plugin::map().keys().cloned().collect();

    languages
        .iter()
        .map(|lang| {
            let language_name = get_language_map().get(lang).unwrap_or(lang).to_string();
            if supported_languages.contains(lang) {
                Language::Supported(language_name)
            } else {
                Language::Unsupported(language_name)
            }
        })
        .collect()
}
