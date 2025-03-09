use std::path::Path;

use log::{error, info};
use test_log::test;

macro_rules! cmd {
    ($program:expr, $( $arg:expr ),*) => {{
        use std::process::Command;
        let mut command = Command::new($program);
        command.args(&[$($arg),*]);
        command
    }};
}

fn flint() -> std::path::PathBuf {
    use std::sync::OnceLock;
    static FLINT_PATH: OnceLock<std::path::PathBuf> = OnceLock::new();

    FLINT_PATH
        .get_or_init(|| {
            let path = std::env::var("PROJECT_DIR").unwrap();
            let path = Path::new(&path);
            let flint_path = Path::new(&path).join("..").join("flint").join("Cargo.toml");

            let build = cmd![
                "cargo",
                "build",
                "--manifest-path",
                flint_path.to_str().unwrap()
            ]
            .output()
            .expect("Unable to build flint");
            assert!(build.status.success());

            let flint_bin = path.join("../target/debug/flint-core");

            flint_bin
        })
        .clone()
}

/// Sets the current working directory to the specified path.
///
/// # Arguments
///
/// * `path` - A path to set as the current working directory
///
/// # Returns
///
/// The previous working directory path

#[test]
fn test_config_generate() {
    let project_dir =
        std::env::var("PROJECT_DIR").expect("PROJECT_DIR environment variable not set");

    info!("{}", project_dir);
    let js_path = Path::new(&project_dir).join("src").join("js");

    info!("{:#?}", js_path);
    info!("{:#?}", js_path.exists());

    let output = std::thread::spawn(move || {
        std::env::set_current_dir(js_path).unwrap();
        cmd![flint(), "generate"]
            .output()
            .expect("Failed to execute Flint CLI")
    })
    .join()
    .unwrap();

    info!("{:#?}", output);

    assert!(output.status.success());
}
