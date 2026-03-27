use tempfile::tempdir;
use zongflow::core::DocumentScanner;
use zongflow::core::SettingsManager;
use zongflow::database::Database;

#[test]
fn test_scan_directory_inserts_into_db() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("test.txt"), "content").unwrap();

    let db_path = dir.path().join("test.db");
    let db = Database::new_with_path(db_path).unwrap();

    let docs = DocumentScanner::scan_directory(dir.path(), &db).unwrap();
    assert_eq!(docs.len(), 1);

    // Verify document is in database
    let db_docs = db.list_documents().unwrap();
    assert_eq!(db_docs.len(), 1);
    assert_eq!(db_docs[0].title, "test");
    assert_eq!(db_docs[0].format, "txt");
}

#[test]
fn test_scan_directory_multiple_files() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("a.txt"), "a").unwrap();
    std::fs::write(dir.path().join("b.md"), "# Title").unwrap();
    std::fs::write(dir.path().join("c.epub"), "epub data").unwrap();
    // unsupported extension should be ignored
    std::fs::write(dir.path().join("d.jpg"), "image").unwrap();

    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();
    let docs = DocumentScanner::scan_directory(dir.path(), &db).unwrap();
    assert_eq!(docs.len(), 3);

    let formats: Vec<&str> = docs.iter().map(|d| d.format.as_str()).collect();
    assert!(formats.contains(&"txt"));
    assert!(formats.contains(&"md"));
    assert!(formats.contains(&"epub"));
}

#[test]
fn test_settings_manager_persistence() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();
    let settings_path = dir.path().join("settings.toml");

    {
        let mut mgr = SettingsManager::new_with_settings_path(db.clone(), settings_path.clone());
        mgr.set_language("ja_JP").unwrap();
        mgr.set_dark_mode(true).unwrap();
        // drop mgr
    }

    // Reload settings from file
    let mgr2 = SettingsManager::new_with_settings_path(db, settings_path);
    assert_eq!(mgr2.get_language(), "ja_JP");
    assert_eq!(mgr2.get_dark_mode(), true);
}

#[test]
fn test_database_duplicate_path_fails() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();

    db.insert_document("Doc1", None, "txt", "/same/path.txt", None)
        .unwrap();
    let result = db.insert_document("Doc2", None, "md", "/same/path.txt", None);
    assert!(result.is_err());
}

#[test]
fn test_i18n_translate_format() {
    // This test just ensures no panics occur; actual translation depends on .mo files
    let result = zongflow::i18n::translate_format("Hello { $name }", &[("name", "World")]);
    // If the key isn't translated, it returns the key as-is
    // The placeholder should still be replaced
    assert!(result.contains("World"));
}

// Workflow integration tests

#[test]
fn test_full_scan_list_delete_workflow() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();

    // Create test files
    std::fs::write(dir.path().join("doc1.txt"), "content1").unwrap();
    std::fs::write(dir.path().join("doc2.md"), "# Title").unwrap();

    // Scan directory
    let docs = DocumentScanner::scan_directory(dir.path(), &db).unwrap();
    assert_eq!(docs.len(), 2);

    // List documents
    let listed = db.list_documents().unwrap();
    assert_eq!(listed.len(), 2);

    // Delete one document
    db.delete_document(&listed[0].path).unwrap();

    // Verify only one remains
    let after_delete = db.list_documents().unwrap();
    assert_eq!(after_delete.len(), 1);
}

#[test]
fn test_settings_change_verify_persistence() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();
    let settings_path = dir.path().join("settings.toml");

    // Initial settings
    {
        let mgr = SettingsManager::new_with_settings_path(db.clone(), settings_path.clone());
        assert_eq!(mgr.get_language(), "en_US");
        assert_eq!(mgr.get_dark_mode(), false);
    }

    // Change settings
    {
        let mut mgr = SettingsManager::new_with_settings_path(db.clone(), settings_path.clone());
        mgr.set_language("zh_CN").unwrap();
        mgr.set_dark_mode(true).unwrap();
        mgr.set_output_folder(&dir.path().join("output")).unwrap();
    }

    // Verify persistence
    {
        let mgr = SettingsManager::new_with_settings_path(db.clone(), settings_path.clone());
        assert_eq!(mgr.get_language(), "zh_CN");
        assert_eq!(mgr.get_dark_mode(), true);
        assert_eq!(mgr.get_output_folder(), dir.path().join("output"));
    }

    // Reset settings
    {
        let mut mgr = SettingsManager::new_with_settings_path(db.clone(), settings_path.clone());
        mgr.reset_settings().unwrap();
    }

    // Verify reset
    {
        let mgr = SettingsManager::new_with_settings_path(db, settings_path);
        assert_eq!(mgr.get_language(), "en_US");
        assert_eq!(mgr.get_dark_mode(), false);
    }
}

#[test]
fn test_multiple_scan_passes_idempotent() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();

    std::fs::write(dir.path().join("doc.txt"), "content").unwrap();

    // First scan
    let docs1 = DocumentScanner::scan_directory(dir.path(), &db).unwrap();
    assert_eq!(docs1.len(), 1);
    let id1 = docs1[0].id;

    // Second scan - should find existing document, not insert duplicate
    let docs2 = DocumentScanner::scan_directory(dir.path(), &db).unwrap();
    assert_eq!(docs2.len(), 1);
    assert_eq!(docs2[0].id, id1);

    // Verify only one document in database
    let all_docs = db.list_documents().unwrap();
    assert_eq!(all_docs.len(), 1);
}

#[test]
fn test_database_clear_and_rescan() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();

    // Create files and scan
    std::fs::write(dir.path().join("doc1.txt"), "content1").unwrap();
    std::fs::write(dir.path().join("doc2.md"), "content2").unwrap();
    DocumentScanner::scan_directory(dir.path(), &db).unwrap();
    assert_eq!(db.list_documents().unwrap().len(), 2);

    // Clear database
    db.clear_documents().unwrap();
    assert_eq!(db.list_documents().unwrap().len(), 0);

    // Rescan - should re-add documents
    let docs = DocumentScanner::scan_directory(dir.path(), &db).unwrap();
    assert_eq!(docs.len(), 2);
    assert_eq!(db.list_documents().unwrap().len(), 2);
}

#[test]
fn test_output_folder_settings_workflow() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();
    let settings_path = dir.path().join("settings.toml");
    let output_dir = dir.path().join("my_output");

    // Set output folder
    {
        let mut mgr = SettingsManager::new_with_settings_path(db.clone(), settings_path.clone());
        mgr.set_output_folder(&output_dir).unwrap();
    }

    // Verify it persists
    {
        let mgr = SettingsManager::new_with_settings_path(db, settings_path);
        assert_eq!(mgr.get_output_folder(), output_dir);
    }
}

#[test]
fn test_document_with_unicode_metadata() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();

    // Insert document with unicode title and author
    let id = db
        .insert_document(
            "日本語ドキュメント",
            Some("著者名"),
            "txt",
            "/path/to/日本語.txt",
            None,
        )
        .unwrap();

    let doc = db
        .get_document_by_path("/path/to/日本語.txt")
        .unwrap()
        .unwrap();
    assert_eq!(doc.title, "日本語ドキュメント");
    assert_eq!(doc.author, Some("著者名".to_string()));
    assert_eq!(doc.id, id);
}
