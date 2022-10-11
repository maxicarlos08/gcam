use crate::{cam_thread::camera_loop::CameraThreadState, ui::state::AppState};
use epaint::ColorImage;
use gcam_lib::error::{AppError, AppResult};
use std::fmt;

pub type FromCameraThreadClosure = Box<dyn FnOnce(&mut AppState) + Send>;
pub type ToCameraThreadClosure =
  Box<dyn FnOnce(&mut CameraThreadState) -> AppResult<FromCameraThreadClosure> + Send>;

#[derive(PartialEq)]
pub struct PreviewImage(pub ColorImage);

pub enum MessageFromThread {
  PreviewCapture(PreviewImage),
  Closure(FromCameraThreadClosure),
  Error(AppError),
}

pub enum MessageToThread {
  SetLiveView(bool),
  Closure(ToCameraThreadClosure),
  Break,
}

impl fmt::Debug for PreviewImage {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "PreviewImage")
  }
}
