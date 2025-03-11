use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::get_flag;

#[derive(Copy, Clone, Debug, Default)]
pub enum LogKind {
    #[default]
    Info,
    Success,
    Error,
    Warn,
    Debug,
}

pub static LOGS: RwLock<Vec<(LogKind, String)>> = RwLock::new(vec![]);

pub fn get_logs() -> Result<
    RwLockReadGuard<'static, Vec<(LogKind, String)>>,
    std::sync::PoisonError<RwLockReadGuard<'static, Vec<(LogKind, String)>>>,
> {
    LOGS.read()
}

pub fn get_logs_mut() -> Result<
    RwLockWriteGuard<'static, Vec<(LogKind, String)>>,
    std::sync::PoisonError<RwLockWriteGuard<'static, Vec<(LogKind, String)>>>,
> {
    LOGS.write()
}

pub fn add_log(kind: LogKind, message: String) {
    use std::fs::OpenOptions;
    use std::io::Write;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs.txt")
        .unwrap();
    let prefix = match kind {
        LogKind::Info => "[info]:",
        LogKind::Success => "[success]:",
        LogKind::Error => "[error]:",
        LogKind::Warn => "[warn]:",
        LogKind::Debug => "[debug]:",
    };

    let is_non_interactive = get_flag!(non_interactive);

    let log = format!("{} {}", prefix, message);
    if is_non_interactive {
        println!("{}", log);
    }
    writeln!(file, "{}", log).unwrap();
    get_logs_mut().unwrap().push((kind, log));
}

#[macro_export]
macro_rules! log {
    ($kind:expr, $($arg:tt)*) => {{
        let message = format!($($arg)*);
        $crate::util::logs::add_log($kind, message);
    }};
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        $crate::log!(crate::util::logs::LogKind::Info, $($arg)*);
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        $crate::log!(crate::util::logs::LogKind::Warn, $($arg)*);
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        $crate::log!(crate::util::logs::LogKind::Error, $($arg)*);
    }};
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        $crate::log!(crate::util::logs::LogKind::Debug, $($arg)*);
    }};
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {{
        $crate::log!(crate::util::logs::LogKind::Success, $($arg)*);
    }};
}
