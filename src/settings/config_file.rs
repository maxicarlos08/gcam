use directories::ProjectDirs;
use gcam_lib::error::AppResult;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs};

const APP_ORG: &str = "maxicarlos08";
const APP_NAME: &str = "GCam";

const CONFIG_FILENAME: &str = "config.yaml";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DevSettings {
    pub exclude_settings: HashSet<String>,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    pub dev_settings: DevSettings,
}

impl Default for DevSettings {
    fn default() -> Self {
        Self {
            exclude_settings: HashSet::from(
                include!("defaults/exclude_cam_settings").map(Into::into),
            ),
        }
    }
}

impl Settings {
    pub fn project_directories() -> Option<ProjectDirs> {
        ProjectDirs::from("com", APP_ORG, APP_NAME)
    }

    pub fn get_user_settings() -> AppResult<Option<Self>> {
        if let Some(project_dirs) = Self::project_directories() {
            let config = project_dirs.config_dir().join(CONFIG_FILENAME);

            if config.is_file() {
                match serde_yaml::from_str::<Settings>(&fs::read_to_string(config)?) {
                    Ok(settings) => Ok(Some(settings)),
                    Err(err) => Err(format!("Failed to parse configuration file: {}", err))?,
                }
            } else {
                Err("Configuration file should be a file")?
            }
        } else {
            Ok(None)
        }
    }

    pub fn save(&self) -> AppResult<()> {
        if let Some(project_dirs) = Self::project_directories() {
            fs::create_dir_all(project_dirs.config_dir())?;
            let config_file = fs::File::create(project_dirs.config_dir().join(CONFIG_FILENAME))?;

            serde_yaml::to_writer(config_file, self)
                .map_err(|err| format!("Failed to write configuration file: {}", err))?;

            Ok(())
        } else {
            Err("Could not get project config directory")?
        }
    }
}
