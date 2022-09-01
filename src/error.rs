use crate::app::AppState;
use gcam_lib::error::{AppError, AppResult};

pub trait CatchAppResult<V> {
    fn catch(self, state: &mut AppState) -> AppResult<V>;
}

pub trait ToUIError {
    fn to_ui_error(&self) -> UiError;
}

#[derive(Clone, Hash)]
pub struct UiError {
    pub title: &'static str,
    pub message: String,
}

impl<T> CatchAppResult<T> for AppResult<T> {
    fn catch(self, state: &mut AppState) -> AppResult<T> {
        match self {
            Ok(v) => Ok(v),
            Err(err) => {
                state.show_error(err.to_ui_error());
                Err(err)
            }
        }
    }
}

impl ToUIError for AppError {
    fn to_ui_error(&self) -> UiError {
        UiError {
            title: self.title(),
            message: self.to_string(),
        }
    }
}
