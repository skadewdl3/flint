use std::{error::Error as ErrorTrait, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Deserialization error: {0}")]
    Deserialization(#[from] toml::de::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::ser::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Application error: {0}")]
    Err(String),

    #[error("User requested exit")]
    Exit,

    #[error("Lua error: {0}")]
    LuaError(#[from] mlua::Error),

    #[error("Environment error: {0}")]
    Env(#[from] dotenvy::Error),
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

// Macro to create AppError::Err with format string
#[macro_export]
macro_rules! app_err {
    ($($arg:tt)*) => {{
        let error_msg = format!($($arg)*);
        $crate::error!("{}", error_msg);
        Err($crate::error::AppError::Err(error_msg))
    }};
}

// Create type alias for Result with AppError as default error type
pub type AppResult<T> = Result<T, AppError>;

impl From<AppError> for mlua::Error {
    fn from(err: AppError) -> Self {
        mlua::Error::ExternalError(Arc::new(err))
    }
}
