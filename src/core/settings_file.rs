use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub dark_mode: bool,
    #[serde(default = "default_output_folder")]
    pub output_folder: PathBuf,
}

fn default_language() -> String {
    "en_US".to_string()
}

fn default_output_folder() -> PathBuf {
    dirs::download_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: default_language(),
            dark_mode: false,
            output_folder: default_output_folder(),
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            tracing::info!(path = ?path, "Settings file not found, using defaults");
            return Ok(Settings::default());
        }
        tracing::debug!(path = ?path, "Loading settings file");
        let content = fs::read_to_string(path).with_context(|| {
            tracing::error!(path = ?path, "Failed to read settings file");
            format!("Failed to read settings file: {:?}", path)
        })?;
        let settings: Settings = toml::from_str(&content).with_context(|| {
            tracing::error!(path = ?path, "Failed to parse settings file");
            format!("Failed to parse settings file: {:?}", path)
        })?;
        tracing::info!(path = ?path, language = %settings.language, dark_mode = settings.dark_mode, "Settings loaded successfully");
        Ok(settings)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        tracing::debug!(path = ?path, language = %self.language, dark_mode = self.dark_mode, "Saving settings");
        let content = toml::to_string_pretty(self).with_context(|| {
            tracing::error!("Failed to serialize settings");
            "Failed to serialize settings"
        })?;
        fs::write(path, content).with_context(|| {
            tracing::error!(path = ?path, "Failed to write settings file");
            format!("Failed to write settings file: {:?}", path)
        })?;
        tracing::info!(path = ?path, "Settings saved successfully");
        Ok(())
    }
}
