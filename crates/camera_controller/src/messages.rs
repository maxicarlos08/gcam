use crate::settings::CameraSettings;
use epaint::ColorImage;
use gcam_lib::error::AppError;
use gphoto2::filesys::StorageInfo;

#[derive(PartialEq)]
pub struct PreviewImage(pub ColorImage);

#[derive(Debug, PartialEq)]
pub enum CameraCommandResponse {
    CameraConfig(CameraSettings),
    LiveView(bool),
}

#[derive(Debug, PartialEq)]
pub enum MessageFromThread {
    PreviewCapture(PreviewImage),
    CameraCommandResponse(CameraCommandResponse),
    CameraList(Vec<(String, String)>),
    CameraOpen(Option<CameraInfo>),
    Error(AppError),
}

#[derive(Debug)]
pub struct CameraInfo {
    pub model: String,
    pub port: String,
    pub manual: Option<String>,
    pub summary: Option<String>,
    pub about: Option<String>,
    pub abilities: gphoto2::abilities::Abilities,
    pub storages: Vec<StorageInfo>,
}

#[derive(Debug, PartialEq)]
pub enum CameraCommand {
    SetLiveView(bool),
    GetConfig,
    SetConfig(CameraSettings),
}

#[derive(Debug, PartialEq)]
pub enum MessageToThread {
    ListCameras,
    CameraCommand(CameraCommand),
    UseCamera(String, String),
    CloseCamera,
    Break,
}

impl PartialEq for CameraInfo {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl std::fmt::Debug for PreviewImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PreviewImage")
    }
}
