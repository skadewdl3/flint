use ignore::Walk;
use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::sync::OnceLock;

#[allow(unused)]
struct FileTypeDetails<'a> {
    name: &'a str,
    linters: Vec<&'a str>,
    testers: Vec<&'a str>,
}

static LANGUAGE_MAP: OnceLock<HashMap<&'static str, FileTypeDetails<'static>>> = OnceLock::new();
static SUPPORTED_LANGUAGES: OnceLock<BTreeSet<&'static str>> = OnceLock::new();
fn get_language_map() -> &'static HashMap<&'static str, FileTypeDetails<'static>> {
    LANGUAGE_MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("rs", FileTypeDetails::new("Rust", vec![], vec![]));
        m.insert("py", FileTypeDetails::new("Python", vec![], vec![]));
        m.insert(
            "js",
            FileTypeDetails::new("JavaScript", vec!["eslint"], vec!["jest"]),
        );
        m.insert(
            "ts",
            FileTypeDetails::new("TypeScript", vec!["eslint"], vec!["jest"]),
        );
        m.insert("java", FileTypeDetails::new("Java", vec![], vec![]));
        m.insert("cpp", FileTypeDetails::new("C++", vec![], vec![]));
        m.insert("cc", FileTypeDetails::new("C++", vec![], vec![]));
        m.insert("cxx", FileTypeDetails::new("C++", vec![], vec![]));
        m.insert("c", FileTypeDetails::new("C", vec![], vec![]));
        m.insert("go", FileTypeDetails::new("Go", vec![], vec![]));
        m.insert("rb", FileTypeDetails::new("Ruby", vec![], vec![]));
        m.insert("php", FileTypeDetails::new("PHP", vec![], vec![]));
        m.insert("swift", FileTypeDetails::new("Swift", vec![], vec![]));
        m.insert("kt", FileTypeDetails::new("Kotlin", vec![], vec![]));
        m.insert("kts", FileTypeDetails::new("Kotlin", vec![], vec![]));
        m.insert("cs", FileTypeDetails::new("C#", vec![], vec![]));
        m.insert("toml", FileTypeDetails::new("Toml", vec![], vec![]));
        m
    })
}

pub fn get_supported_languages() -> &'static BTreeSet<&'static str> {
    SUPPORTED_LANGUAGES.get_or_init(|| {
        let mut s = BTreeSet::new();
        s.insert("JavaScript");
        s.insert("TypeScript");
        s.insert("C#");
        s.insert("SQL");
        s
    })
}

impl<'a> FileTypeDetails<'a> {
    pub fn new(name: &'a str, linters: Vec<&'a str>, testers: Vec<&'a str>) -> Self {
        return Self {
            name,
            linters,
            testers,
        };
    }
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
                        if let Some(language_details) =
                            get_language_map().get(ext.to_lowercase().as_str())
                        {
                            languages.insert(language_details.name.to_string());
                        }
                    }
                }
            }
        }
    }
    let supported_languages = get_supported_languages();
    let unsupported_languages: BTreeSet<String> = languages
        .iter()
        .filter(|lang| !supported_languages.contains(lang.as_str()))
        .cloned()
        .collect();
    (languages, unsupported_languages)
}
