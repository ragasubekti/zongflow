use chrono::{Duration, Utc};
use tempfile::tempdir;
use zongflow::database::Database;

#[test]
fn test_database_creation() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::new_with_path(db_path).unwrap();
    // Tables should be created; we can verify by inserting data
    db.set_setting("test_key", "test_value").unwrap();
    assert_eq!(
        db.get_setting("test_key").unwrap(),
        Some("test_value".to_string())
    );
}

#[test]
fn test_settings_crud() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::new_with_path(db_path).unwrap();

    // Set a setting
    db.set_setting("language", "en-US").unwrap();
    assert_eq!(
        db.get_setting("language").unwrap(),
        Some("en-US".to_string())
    );

    // Update setting
    db.set_setting("language", "zh-CN").unwrap();
    assert_eq!(
        db.get_setting("language").unwrap(),
        Some("zh-CN".to_string())
    );

    // Delete setting
    db.delete_setting("language").unwrap();
    assert_eq!(db.get_setting("language").unwrap(), None);

    // Clear all settings
    db.set_setting("key1", "val1").unwrap();
    db.set_setting("key2", "val2").unwrap();
    db.clear_settings().unwrap();
    assert_eq!(db.get_setting("key1").unwrap(), None);
    assert_eq!(db.get_setting("key2").unwrap(), None);
}

#[test]
fn test_document_crud() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::new_with_path(db_path).unwrap();

    // Insert a document
    let doc_id = db
        .insert_document("Test Doc", Some("Author"), "txt", "/path/to/doc.txt", None)
        .unwrap();
    assert!(doc_id > 0);

    // Retrieve by path
    let doc = db
        .get_document_by_path("/path/to/doc.txt")
        .unwrap()
        .unwrap();
    assert_eq!(doc.title, "Test Doc");
    assert_eq!(doc.author, Some("Author".to_string()));
    assert_eq!(doc.format, "txt");

    // Update last opened
    db.update_document_last_opened("/path/to/doc.txt").unwrap();
    let doc2 = db
        .get_document_by_path("/path/to/doc.txt")
        .unwrap()
        .unwrap();
    assert!(doc2.last_opened.is_some());

    // List documents
    let docs = db.list_documents().unwrap();
    assert_eq!(docs.len(), 1);

    // Delete document
    db.delete_document("/path/to/doc.txt").unwrap();
    assert_eq!(db.get_document_by_path("/path/to/doc.txt").unwrap(), None);

    // Clear all documents
    db.insert_document("Doc1", None, "md", "/doc1.md", None)
        .unwrap();
    db.insert_document("Doc2", None, "epub", "/doc2.epub", None)
        .unwrap();
    db.clear_documents().unwrap();
    assert_eq!(db.list_documents().unwrap().len(), 0);
}

#[test]
fn test_duplicate_path_insert_fails() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();

    db.insert_document("Doc1", None, "txt", "/same/path.txt", None)
        .unwrap();
    let result = db.insert_document("Doc2", None, "md", "/same/path.txt", None);
    assert!(result.is_err());
}

#[test]
fn test_insert_document_with_all_none_optionals() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();

    let id = db
        .insert_document("Minimal Doc", None, "txt", "/minimal.txt", None)
        .unwrap();
    assert!(id > 0);

    let doc = db.get_document_by_path("/minimal.txt").unwrap().unwrap();
    assert_eq!(doc.title, "Minimal Doc");
    assert_eq!(doc.author, None);
    assert_eq!(doc.cover_path, None);
}

#[test]
fn test_update_last_opened_nonexistent_path_succeeds() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();

    // Should succeed with 0 rows affected
    let result = db.update_document_last_opened("/nonexistent.txt");
    assert!(result.is_ok());
}

#[test]
fn test_clone_shares_connection() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();
    let db_clone = db.clone();

    // Insert via original
    db.insert_document("Shared", None, "txt", "/shared.txt", None)
        .unwrap();

    // Retrieve via clone
    let doc = db_clone.get_document_by_path("/shared.txt").unwrap();
    assert!(doc.is_some());
    assert_eq!(doc.unwrap().title, "Shared");
}

#[test]
fn test_list_documents_ordering() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();

    // Insert three documents with different date_added
    let now = Utc::now();
    let yesterday = now - Duration::days(1);

    // Insert documents
    let _id1 = db
        .insert_document("Doc1", None, "txt", "/doc1.txt", None)
        .unwrap();
    let id2 = db
        .insert_document("Doc2", None, "txt", "/doc2.txt", None)
        .unwrap();
    let id3 = db
        .insert_document("Doc3", None, "txt", "/doc3.txt", None)
        .unwrap();

    // Update last_opened for doc2 and doc3 (doc1 remains NULL)
    // Use explicit timestamps via raw SQL for testing
    db.set_last_opened_for_test(id2, &yesterday.to_rfc3339())
        .unwrap();
    db.set_last_opened_for_test(id3, &now.to_rfc3339()).unwrap();

    // List documents; order should be: doc3 (most recent last_opened), doc2 (older last_opened), doc1 (NULL last_opened, ordered by date_added DESC)
    let docs = db.list_documents().unwrap();
    assert_eq!(docs.len(), 3);
    assert_eq!(docs[0].title, "Doc3");
    assert_eq!(docs[1].title, "Doc2");
    assert_eq!(docs[2].title, "Doc1");
}

#[test]
fn test_unicode_characters_in_document() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();

    let id = db
        .insert_document(
            "日本語ドキュメント",
            Some("著者名"),
            "txt",
            "/path/to/日本語.txt",
            None,
        )
        .unwrap();
    assert!(id > 0);

    let doc = db
        .get_document_by_path("/path/to/日本語.txt")
        .unwrap()
        .unwrap();
    assert_eq!(doc.title, "日本語ドキュメント");
    assert_eq!(doc.author, Some("著者名".to_string()));
}

#[test]
fn test_empty_string_values() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();

    // Empty title should work
    let id = db
        .insert_document("", None, "txt", "/empty_title.txt", None)
        .unwrap();
    assert!(id > 0);

    let doc = db
        .get_document_by_path("/empty_title.txt")
        .unwrap()
        .unwrap();
    assert_eq!(doc.title, "");
    assert_eq!(doc.author, None);
}

#[test]
fn test_setting_empty_key_value() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();

    // Empty key should work
    db.set_setting("", "value").unwrap();
    assert_eq!(db.get_setting("").unwrap(), Some("value".to_string()));

    // Empty value should work
    db.set_setting("key", "").unwrap();
    assert_eq!(db.get_setting("key").unwrap(), Some("".to_string()));
}

#[test]
fn test_document_with_long_path() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();

    let long_path = "/very/long/path/".repeat(50) + "document.txt";
    let id = db
        .insert_document("Long Path Doc", None, "txt", &long_path, None)
        .unwrap();
    assert!(id > 0);

    let doc = db.get_document_by_path(&long_path).unwrap().unwrap();
    assert_eq!(doc.path, long_path);
}

#[test]
fn test_delete_nonexistent_document_warns() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();

    // Delete nonexistent document should succeed (0 rows affected)
    let result = db.delete_document("/nonexistent.txt");
    assert!(result.is_ok());
}

#[test]
fn test_multiple_settings_operations() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();

    // Set multiple settings
    for i in 0..100 {
        db.set_setting(&format!("key{}", i), &format!("value{}", i))
            .unwrap();
    }

    // Verify all settings
    for i in 0..100 {
        assert_eq!(
            db.get_setting(&format!("key{}", i)).unwrap(),
            Some(format!("value{}", i))
        );
    }

    // Clear all settings
    db.clear_settings().unwrap();

    // Verify all settings are gone
    for i in 0..100 {
        assert_eq!(db.get_setting(&format!("key{}", i)).unwrap(), None);
    }
}

#[test]
fn test_document_with_special_characters_in_path() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();

    let special_path = "/path/with spaces/and-dashes_under.txt";
    let id = db
        .insert_document("Special Path", None, "txt", special_path, None)
        .unwrap();
    assert!(id > 0);

    let doc = db.get_document_by_path(special_path).unwrap().unwrap();
    assert_eq!(doc.path, special_path);
}
