use crate::app::AppState;
use std::{error, fmt, ops::ControlFlow};

pub type AppResult<T> = Result<T, AppError>;

pub trait CatchAppResult<B, C> {
    fn catch(self, state: &mut AppState) -> ControlFlow<B, C>;
}

#[derive(Clone, Hash)]
pub struct UiError {
    pub title: &'static str,
    pub message: String,
}

#[derive(Debug)]
pub enum AppError {
    GPhotoError(gphoto2::Error),
    Unknown(String),
    IoError(std::io::Error),
    ThreadError(String),
}

impl AppError {
    pub fn title(&self) -> &'static str {
        match self {
            Self::GPhotoError(_) => "Gphoto Error",
            Self::Unknown(_) => "Unknown Error",
            Self::IoError(_) => "I/O Error",
            Self::ThreadError(_) => "Threading error",
        }
    }

    pub fn ui_error(&self) -> UiError {
        UiError {
            title: self.title(),
            message: self.to_string(),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GPhotoError(err) => {
                write!(f, "An error has ocurred in the gphoto2 backend: {:?}", err)
            }
            Self::Unknown(err) => write!(f, "An unknown error has ocurred: {}", err),
            Self::ThreadError(err) => write!(f, "An error threading error has ocurred: {}", err),
            AppError::IoError(err) => write!(f, "An I/O error has ocurred: {:?}", err),
        }
    }
}

impl error::Error for AppError {}

impl<T> CatchAppResult<AppError, T> for AppResult<T> {
    fn catch(self, state: &mut AppState) -> ControlFlow<AppError, T> {
        match self {
            Ok(v) => ControlFlow::Continue(v),
            Err(err) => {
                state.show_error(err.ui_error());
                ControlFlow::Break(err)
            }
        }
    }
}

impl From<gphoto2::Error> for AppError {
    fn from(err: gphoto2::Error) -> Self {
        Self::GPhotoError(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        Self::Unknown(err)
    }
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        Self::Unknown(err.to_string())
    }
}

impl<T> From<std::sync::mpsc::SendError<T>> for AppError {
    fn from(err: std::sync::mpsc::SendError<T>) -> Self {
        Self::ThreadError(err.to_string())
    }
}
