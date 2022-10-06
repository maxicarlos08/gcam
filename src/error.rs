use std::ops::Deref;

use crate::ui::state::AppState;
use gcam_lib::error::{AppError, AppResult};

pub struct CaughtAppResult<T> {
    result: AppResult<T>,
}

pub trait CatchAppResult<V> {
    fn catch(self, state: &mut AppState) -> CaughtAppResult<V>;
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
    fn catch(self, state: &mut AppState) -> CaughtAppResult<T> {
        CaughtAppResult {
            result: match self {
                Ok(v) => Ok(v),
                Err(err) => {
                    state.show_error(err.to_ui_error());
                    Err(err)
                }
            },
        }
    }
}

impl ToUIError for AppError {
    fn to_ui_error(&self) -> UiError {
        UiError { title: self.title(), message: self.to_string() }
    }
}

impl<T> Deref for CaughtAppResult<T> {
    type Target = AppResult<T>;

    fn deref(&self) -> &Self::Target {
        &self.result
    }
}
