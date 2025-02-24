pub use app::*;

pub mod app;
pub mod generate;
pub mod help;
pub mod init;
pub mod test;

use crossterm::event::Event;
use ratatui::widgets::WidgetRef;
use std::io;
use thiserror::Error;

pub trait AppWidget: WidgetRef {
    fn setup(&mut self) -> AppResult<()> {
        Ok(())
    }
    fn handle_events(&mut self, _event: Event) -> AppResult<()> {
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Deserialization error: {0}")]
    Deserialization(#[from] toml::de::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::ser::Error),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Application error: {0}")]
    Err(String),

    #[error("User requested exit")]
    Exit,
}

// Create type alias for Result with AppError as default error type
pub type AppResult<T> = Result<T, AppError>;
