pub use app::*;

pub mod app;
pub mod generate;
pub mod help;
pub mod init;
pub mod install;
pub mod test;

use crossterm::event::Event;
use ratatui::widgets::WidgetRef;
use std::error::Error as ErrorTrait;
use std::io;
use std::sync::mpsc::Sender;
use thiserror::Error;

pub trait AppWidget: WidgetRef {
    fn setup(&mut self) -> AppResult<()> {
        Ok(())
    }
    fn handle_events(&mut self, _event: Event) -> AppResult<()> {
        Ok(())
    }
    fn set_exit_sender(&mut self, _exit_sender: Sender<()>) {}
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

// Convert Box<dyn Error> to AppError using a catch-all approach
impl From<Box<dyn ErrorTrait>> for AppError {
    fn from(error: Box<dyn ErrorTrait>) -> Self {
        // Try to downcast to AppError first
        let str_err = error.to_string();
        if let Ok(app_error) = error.downcast::<AppError>() {
            return *app_error;
        } else {
            return AppError::Err(str_err);
        }
    }
}

// Create type alias for Result with AppError as default error type
pub type AppResult<T> = Result<T, AppError>;
