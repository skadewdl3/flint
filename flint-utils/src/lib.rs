pub mod env;
pub mod error;
pub mod flags;
pub mod logs;

pub use error::{AppError as Error, AppResult as Result};

#[macro_export]
macro_rules! cmd {
    ($program:expr, $($arg:expr),* $(,)?) => {{
        let mut command = std::process::Command::new($program);
        $(command.arg($arg);)*
        command
    }};
}
