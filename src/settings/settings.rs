use directories::ProjectDirs;
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

    pub fn get_user_settings() -> Result<Option<Self>, String> {
        if let Some(project_dirs) = Self::project_directories() {
            let config = project_dirs.config_dir().join(CONFIG_FILENAME);

            if config.is_file() {
                match fs::read_to_string(config)
                    .map(|file_content| serde_yaml::from_str::<Settings>(&file_content))
                {
                    Ok(Ok(settings)) => Ok(Some(settings)),
                    Err(err) => Err(format!("Failed to read configuration file: {}", err)),
                    Ok(Err(err)) => Err(format!("Failed to parse configuration file: {}", err)),
                }
            } else {
                Err("Configuration file should be a file".into())
            }
        } else {
            Ok(None)
        }
    }

    pub fn write_default() -> Result<Self, String> {
        if let Some(project_dirs) = Self::project_directories() {
            let config_file = fs::File::create(project_dirs.config_dir().join(CONFIG_FILENAME))
                .map_err(|err| format!("Could not create configuration file: {}", err))?;
            let config = Self::default();

            serde_yaml::to_writer(config_file, &config)
                .map_err(|err| format!("Failed to write configuration file: {}", err))?;

            Ok(config)
        } else {
            Err("Could not get project config directory".into())
        }
    }
}
