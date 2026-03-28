use anyhow::Result;
use tempfile::tempdir;
use zongflow::core::SettingsManager;
use zongflow::database::Database;

#[test]
fn test_database_connection_error_handling() {
    // Test that database operations return proper error types
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::new_with_path(db_path).unwrap();

    // Insert a document
    let id = db
        .insert_document(
            "Test Document",
            Some("Test Author"),
            "txt",
            "/test/path.txt",
            None,
            None,
            None,
        )
        .unwrap();
    assert!(id > 0);

    // Try to insert duplicate path (should fail)
    let result = db.insert_document("Another Document", None, "txt", "/test/path.txt", None, None, None);
    assert!(result.is_err());

    // Error should contain context
    let err = result.unwrap_err();
    let err_string = format!("{}", err);
    assert!(err_string.contains("Failed to insert document"));
}

#[test]
fn test_database_query_error_handling() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();

    // Query for non-existent document should return None, not error
    let result = db.get_document_by_path("/nonexistent/path.txt").unwrap();
    assert!(result.is_none());

    // Delete non-existent document should succeed (0 rows affected)
    let result = db.delete_document("/nonexistent/path.txt");
    assert!(result.is_ok());
}

#[test]
fn test_settings_manager_error_handling() -> Result<()> {
    let dir = tempdir()?;
    let db = Database::new_with_path(dir.path().join("test.db"))?;
    let settings_path = dir.path().join("settings.toml");

    // Create settings manager with non-existent file
    let mut mgr = SettingsManager::new_with_settings_path(db.clone(), settings_path.clone());

    // Getters should work with defaults
    assert_eq!(mgr.get_language(), "en_US");
    assert!(!mgr.get_dark_mode());

    // Set some values
    mgr.set_language("zh_CN")?;
    mgr.set_dark_mode(true)?;

    // Values should be persisted
    assert_eq!(mgr.get_language(), "zh_CN");
    assert!(mgr.get_dark_mode());

    // Create new manager with same database
    let mgr2 = SettingsManager::new_with_settings_path(db, settings_path);
    assert_eq!(mgr2.get_language(), "zh_CN");
    assert!(mgr2.get_dark_mode());

    Ok(())
}

#[test]
fn test_database_edge_cases() -> Result<()> {
    let dir = tempdir()?;
    let db = Database::new_with_path(dir.path().join("test.db"))?;

    // Test empty values
    db.set_setting("", "")?;
    assert_eq!(db.get_setting("")?, Some("".to_string()));

    // Test very long values
    let long_value = "a".repeat(10000);
    db.set_setting("long_key", &long_value)?;
    assert_eq!(db.get_setting("long_key")?, Some(long_value));

    // Test document with all fields
    let id = db.insert_document(
        "Complete Document",
        Some("Author Name"),
        "epub",
        "/path/to/document.epub",
        Some("/path/to/cover.jpg"),
        None,
        None,
    )?;

    let doc = db.get_document_by_path("/path/to/document.epub")?.unwrap();
    assert_eq!(doc.id, id);
    assert_eq!(doc.title, "Complete Document");
    assert_eq!(doc.author, Some("Author Name".to_string()));
    assert_eq!(doc.format, "epub");
    assert_eq!(doc.path, "/path/to/document.epub");
    assert!(doc.date_added.contains("T")); // RFC3339 format
    assert!(doc.cover_path.is_some());

    // Update last opened
    db.update_document_last_opened("/path/to/document.epub")?;
    let doc = db.get_document_by_path("/path/to/document.epub")?.unwrap();
    assert!(doc.last_opened.is_some());

    // List all documents
    let docs = db.list_documents()?;
    assert_eq!(docs.len(), 1);

    // Clear documents
    db.clear_documents()?;
    let docs = db.list_documents()?;
    assert!(docs.is_empty());

    Ok(())
}

#[test]
fn test_database_clone_behavior() -> Result<()> {
    let dir = tempdir()?;
    let db = Database::new_with_path(dir.path().join("test.db"))?;

    // Insert data with first connection
    db.set_setting("key1", "value1")?;

    // Clone database
    let db_clone = db.clone();

    // Clone should see the same data
    assert_eq!(db_clone.get_setting("key1")?, Some("value1".to_string()));

    // Insert via clone
    db_clone.set_setting("key2", "value2")?;

    // Original should see the new data
    assert_eq!(db.get_setting("key2")?, Some("value2".to_string()));

    // Delete via original
    db.delete_setting("key1")?;

    // Clone should see the deletion
    assert_eq!(db_clone.get_setting("key1")?, None);

    Ok(())
}

#[test]
fn test_settings_file_error_handling() -> Result<()> {
    use std::fs;
    use zongflow::core::Settings;

    let dir = tempdir()?;
    let settings_path = dir.path().join("settings.toml");

    // Test loading non-existent file
    let settings = Settings::load(&settings_path)?;
    assert_eq!(settings.language, "en_US");
    assert!(!settings.dark_mode);

    // Test saving
    let mut settings = Settings::default();
    settings.language = "ja_JP".to_string();
    settings.dark_mode = true;
    settings.save(&settings_path)?;

    // Test loading saved file
    let loaded = Settings::load(&settings_path)?;
    assert_eq!(loaded.language, "ja_JP");
    assert!(loaded.dark_mode);

    // Test invalid TOML
    fs::write(&settings_path, "invalid toml content {{{")?;
    let result = Settings::load(&settings_path);
    assert!(result.is_err());

    // Test partial TOML
    fs::write(&settings_path, "language = \"zh_CN\"")?;
    let settings = Settings::load(&settings_path)?;
    assert_eq!(settings.language, "zh_CN");
    assert!(!settings.dark_mode); // Should use default

    Ok(())
}
