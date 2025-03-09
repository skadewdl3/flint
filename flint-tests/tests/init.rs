use log::{error, info};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Once;

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::builder().is_test(true).try_init().ok();
    });
}

macro_rules! cmd {
    ($program:expr, $( $arg:expr ),*) => {{
        use std::process::Command;
        let mut command = Command::new($program);
        command.args(&[$($arg),*]);
        command
    }};
}

#[test]
fn test_flint_help() {
    init_logger();

    let cwd = std::env::current_dir().expect("Failed to get current directory");
    info!("{:#?}", cwd);
    // let flint_bin = cwd.join("target/debug/flint-core");

    // let stdout = String::from_utf8_lossy(&output.stdout);
    // assert!(stdout.contains("Usage: flint"));
}
