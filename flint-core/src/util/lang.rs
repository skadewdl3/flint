use std::{
    collections::{BTreeSet, HashMap},
    path::Path,
};

use ignore::Walk;

use super::{get_plugin_map, LANGUAGE_MAP};

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

pub fn detect_languages<'a>(
    project_path: impl Into<&'a str>,
) -> (BTreeSet<String>, BTreeSet<String>) {
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

    let supported_languages: BTreeSet<String> = get_plugin_map().keys().cloned().collect();

    // println!("{:#?}", get_plugin_list());

    let unsupported_languages: BTreeSet<String> = languages
        .iter()
        .filter(|lang| !supported_languages.contains(lang.as_str()))
        .map(|lang| get_language_map().get(lang).unwrap_or(lang))
        .cloned()
        .collect();

    let languages: BTreeSet<String> = languages
        .iter()
        .map(|lang| get_language_map().get(lang).unwrap_or(lang))
        .cloned()
        .collect();
    (languages, unsupported_languages)
}
