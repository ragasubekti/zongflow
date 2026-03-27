use crate::core::SettingsManager;
use crate::database::Database;
use tempfile::{tempdir, TempDir};

pub struct TestContext {
    pub db: Database,
    pub db_dir: TempDir,
    pub settings_manager: SettingsManager,
    pub settings_dir: TempDir,
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TestContext {
    pub fn new() -> Self {
        let db_dir = tempdir().unwrap();
        let db_path = db_dir.path().join("test.db");
        let db = Database::new_with_path(db_path).unwrap();

        let settings_dir = tempdir().unwrap();
        let settings_path = settings_dir.path().join("settings.toml");
        let settings_manager = SettingsManager::new_with_settings_path(db.clone(), settings_path);

        Self {
            db,
            db_dir,
            settings_manager,
            settings_dir,
        }
    }
}
