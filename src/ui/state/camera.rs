use epaint::ahash::HashMap;

use crate::{cam_thread::settings::CameraSettings, camera::info::CameraInfo};

pub type ModifiedSettingsMap = HashMap<i32, (i32, CameraSettings)>;

#[derive(Debug)]
pub struct UICamera {
    pub info: CameraInfo,
    pub settings: Option<CameraSettings>,
    pub block_config: bool,
    pub modified_settings: ModifiedSettingsMap,
    pub live_view_enabled: bool,
}

impl UICamera {
    pub fn modify_setting(&mut self, section_id: i32, setting: CameraSettings) {
        self.modified_settings.insert(setting.id, (section_id, setting));
    }

    pub fn discard_settings(&mut self) {
        self.modified_settings.clear();
    }
}
