use std::path::PathBuf;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PersistentSettingsStorage {
    pub enable_shears_update_check_on_startup: bool,
    pub use_loose_selection: bool,
    pub enable_experimental_features: bool,
}

impl Default for PersistentSettingsStorage {
    fn default() -> Self {
        Self {
            enable_shears_update_check_on_startup: true,
            use_loose_selection: false,
            enable_experimental_features: false,
        }
    }
}

impl PersistentSettingsStorage {
    fn get_path() -> PathBuf {
        let settings_file_name = if cfg!(debug_assertions) {
            "UserSettings.debug.toml"
        } else {
            "UserSettings.toml"
        };

        if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "shears") {
            proj_dirs.config_dir().join(settings_file_name)
        } else {
            log::warn!("Failed to get appdata folder");
            PathBuf::from(settings_file_name)
        }
    }

    pub fn load_or_default() -> Self {
        let path = Self::get_path();

        if let Ok(contents) = std::fs::read_to_string(&path)
            && let Ok(loaded) = toml::from_str(&contents)
        {
            log::info!(
                "Loading settings from disk from the following path: {}",
                path.to_string_lossy()
            );
            return loaded;
        }

        log::warn!("Failed loading settings from disk, using defaults...");
        Self::default()
    }

    pub fn save_to_file(&self) {
        let path = Self::get_path();

        if let Some(parent) = path.parent() {
            log::info!("Creating {}", path.to_string_lossy());

            if let Err(e) = std::fs::create_dir_all(parent) {
                log::error!("Failed to create folder: {e}");
            }
        }

        if let Ok(toml_string) = toml::to_string_pretty(&self) {
            log::info!("Saving settings to disk...");
            if let Err(e) = std::fs::write(path, toml_string) {
                log::error!("Failed to save settings to disk: {e}");
            }
        }
    }
}
