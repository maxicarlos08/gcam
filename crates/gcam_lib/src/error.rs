use std::{error, fmt};

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
  GPhotoError(gphoto2::Error),
  Unknown(String),
  IoError(std::io::Error),
  ThreadError(String),
  ImageError(image::ImageError),
}

impl AppError {
  pub fn title(&self) -> &'static str {
    match self {
      Self::GPhotoError(_) => "Gphoto Error",
      Self::Unknown(_) => "Unknown Error",
      Self::IoError(_) => "I/O Error",
      Self::ThreadError(_) => "Threading Error",
      Self::ImageError(_) => "Image Error",
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
      Self::IoError(err) => write!(f, "An I/O error has ocurred: {:?}", err),
      Self::ImageError(err) => write!(f, "Image error: {}", err),
    }
  }
}

impl error::Error for AppError {}

impl PartialEq for AppError {
  fn eq(&self, other: &Self) -> bool {
    match self {
      // These errors don't implement PartialEq
      Self::IoError(_) | Self::ImageError(_) => false,
      _ => self == other,
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

impl From<image::ImageError> for AppError {
  fn from(err: image::ImageError) -> Self {
    Self::ImageError(err)
  }
}

impl<T> From<crossbeam_channel::SendError<T>> for AppError {
  fn from(err: crossbeam_channel::SendError<T>) -> Self {
    Self::ThreadError(err.to_string())
  }
}
