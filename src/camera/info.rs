use gphoto2::filesys::StorageInfo;

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
