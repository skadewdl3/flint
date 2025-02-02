use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

static LANGUAGE_MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

fn get_language_map() -> &'static HashMap<&'static str, &'static str> {
    LANGUAGE_MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("rs", "Rust");
        m.insert("py", "Python");
        m.insert("js", "JavaScript");
        m.insert("ts", "TypeScript");
        m.insert("java", "Java");
        m.insert("cpp", "C++");
        m.insert("cc", "C++");
        m.insert("cxx", "C++");
        m.insert("c", "C");
        m.insert("go", "Go");
        m.insert("rb", "Ruby");
        m.insert("php", "PHP");
        m.insert("swift", "Swift");
        m.insert("kt", "Kotlin");
        m.insert("kts", "Kotlin");
        m.insert("cs", "C#");
        m.insert("toml", "Toml");
        m
    })
}

pub fn detect_languages<'a>(project_path: impl Into<&'a str>) -> BTreeSet<String> {
    let mut languages = BTreeSet::new();
    scan_directory(Path::new(project_path.into()), &mut languages);
    languages
}

fn scan_directory(dir: &Path, languages: &mut BTreeSet<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    scan_directory(&path, languages);
                } else if let Some(extension) = path.extension() {
                    if let Some(ext) = extension.to_str() {
                        if let Some(language) = get_language_map().get(ext.to_lowercase().as_str())
                        {
                            languages.insert(language.to_string());
                        }
                    }
                }
            }
        }
    }
}
