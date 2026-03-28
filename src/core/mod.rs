use crate::database::{Database, Document};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing;

mod settings_file;
pub use settings_file::Settings;

pub struct DocumentScanner;

impl DocumentScanner {
    /// Normalize format name to human-readable form
    pub fn normalize_format(ext: &str) -> String {
        match ext.to_lowercase().as_str() {
            "txt" => "Plain Text".to_string(),
            "md" | "markdown" => "Markdown".to_string(),
            "epub" => "EPUB".to_string(),
            other => other.to_string(),
        }
    }

    pub fn scan_directory(dir: &Path, db: &Database) -> Result<Vec<Document>> {
        let span = tracing::span!(tracing::Level::DEBUG, "scan_directory", dir = ?dir);
        let _enter = span.enter();
        let mut documents = Vec::new();
        if !dir.is_dir() {
            return Ok(documents);
        }
        for entry in fs::read_dir(dir).context("Failed to read directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if matches!(ext_lower.as_str(), "txt" | "epub" | "md" | "markdown") {
                        // Check if already in database
                        if let Some(existing) =
                            db.get_document_by_path(path.to_str().unwrap_or_default())?
                        {
                            documents.push(existing);
                        } else {
                            // Extract metadata from file system
                            let title = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("Unknown")
                                .to_string();
                            let format = Self::normalize_format(&ext_lower);

                            // Get file size from metadata
                            let file_size_bytes = fs::metadata(&path).ok().map(|m| m.len() as i64);

                            // Determine text encoding for text formats
                            let text_encoding = match ext_lower.as_str() {
                                "txt" | "md" | "markdown" => Some("UTF-8".to_string()),
                                _ => None, // Binary formats like EPUB don't have text encoding
                            };

                            let doc = Document {
                                id: 0, // will be assigned by database
                                title,
                                author: Some("Unknown".to_string()),
                                format,
                                path: path.to_str().unwrap_or_default().to_string(),
                                date_added: chrono::Utc::now().to_rfc3339(),
                                last_opened: None,
                                cover_path: None,
                                file_size_bytes,
                                text_encoding,
                            };
                            // Insert into database and retrieve with id
                            let id = db.insert_document(
                                &doc.title,
                                doc.author.as_deref(),
                                &doc.format,
                                &doc.path,
                                doc.cover_path.as_deref(),
                                doc.file_size_bytes,
                                doc.text_encoding.as_deref(),
                            )?;
                            let mut doc_with_id = doc;
                            doc_with_id.id = id;
                            documents.push(doc_with_id);
                        }
                    }
                }
            }
        }
        Ok(documents)
    }
}

pub struct SettingsManager {
    db: Database,
    settings: Settings,
    settings_path: PathBuf,
}

impl SettingsManager {
    pub fn new(db: Database) -> Self {
        let settings_path = Self::settings_path();
        let settings = Settings::load(&settings_path).unwrap_or_default();
        SettingsManager {
            db,
            settings,
            settings_path,
        }
    }

    pub fn new_with_settings_path(db: Database, settings_path: PathBuf) -> Self {
        let settings = Settings::load(&settings_path).unwrap_or_default();
        SettingsManager {
            db,
            settings,
            settings_path,
        }
    }

    fn settings_path() -> PathBuf {
        let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("zongflow");
        std::fs::create_dir_all(&path).ok();
        path.push("settings.toml");
        path
    }

    fn save_settings(&self) -> Result<()> {
        self.settings
            .save(&self.settings_path)
            .context("Failed to save settings file")
    }

    pub fn get_language(&self) -> String {
        // Convert old hyphen format to underscore format for gettext
        self.settings.language.replace('-', "_")
    }

    pub fn set_language(&mut self, lang: &str) -> Result<()> {
        self.settings.language = lang.to_string();
        self.save_settings()
    }

    pub fn get_dark_mode(&self) -> bool {
        self.settings.dark_mode
    }

    pub fn set_dark_mode(&mut self, enabled: bool) -> Result<()> {
        self.settings.dark_mode = enabled;
        self.save_settings()
    }

    pub fn get_output_folder(&self) -> PathBuf {
        self.settings.output_folder.clone()
    }

    pub fn set_output_folder(&mut self, path: &Path) -> Result<()> {
        self.settings.output_folder = path.to_path_buf();
        self.save_settings()
    }

    pub fn get_export_format(&self) -> String {
        self.settings.export_format.clone()
    }

    pub fn set_export_format(&mut self, format: &str) -> Result<()> {
        self.settings.export_format = format.to_string();
        self.save_settings()
    }

    pub fn clear_cache(&self) -> Result<()> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?
            .join("zongflow");

        if cache_dir.exists() {
            std::fs::remove_dir_all(&cache_dir).context("Failed to remove cache directory")?;
            std::fs::create_dir_all(&cache_dir).context("Failed to recreate cache directory")?;
        }

        Ok(())
    }

    pub fn clear_database(&self) -> Result<()> {
        self.db
            .clear_documents()
            .context("Failed to clear documents from database")?;
        // Also clear settings file? Probably not, since settings are separate.
        Ok(())
    }

    pub fn reset_settings(&mut self) -> Result<()> {
        self.settings = Settings::default();
        self.save_settings()?;
        // Also clear database settings table for backward compatibility
        self.db
            .clear_settings()
            .context("Failed to clear settings from database")?;
        Ok(())
    }
}
