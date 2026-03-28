use zongflow::database::Document;
use zongflow::document_object::DocumentObject;

#[test]
fn test_document_object_creation() {
    let doc = DocumentObject::new(
        1,
        "Test Document",
        Some("Author Name"),
        "txt",
        "/path/to/file.txt",
        "2025-05-23T22:44:00+00:00",
        Some("2025-05-24T10:30:00+00:00"),
        Some("/path/to/cover.jpg"),
        Some(1024),
        Some("UTF-8"),
    );

    assert_eq!(doc.id(), 1);
    assert_eq!(doc.title(), "Test Document");
    assert_eq!(doc.author(), Some("Author Name".to_string()));
    assert_eq!(doc.format(), "txt");
    assert_eq!(doc.path(), "/path/to/file.txt");
    assert_eq!(doc.file_size_bytes(), 1024);
    assert_eq!(doc.text_encoding(), Some("UTF-8".to_string()));
}

#[test]
fn test_document_object_format_size() {
    let doc = DocumentObject::new(
        1,
        "Test",
        None,
        "txt",
        "/test.txt",
        "2025-05-23T22:44:00+00:00",
        None,
        None,
        Some(1024),
        None,
    );

    // Test 1 KB
    let formatted = doc.format_size();
    assert!(formatted.contains("KB"));

    // Test 1 MB
    let doc_mb = DocumentObject::new(
        2,
        "Test MB",
        None,
        "txt",
        "/test2.txt",
        "2025-05-23T22:44:00+00:00",
        None,
        None,
        Some(1024 * 1024),
        None,
    );
    let formatted_mb = doc_mb.format_size();
    assert!(formatted_mb.contains("MB"));

    // Test 1 GB
    let doc_gb = DocumentObject::new(
        3,
        "Test GB",
        None,
        "txt",
        "/test3.txt",
        "2025-05-23T22:44:00+00:00",
        None,
        None,
        Some(1024 * 1024 * 1024),
        None,
    );
    let formatted_gb = doc_gb.format_size();
    assert!(formatted_gb.contains("GB"));
}

#[test]
fn test_document_object_format_date() {
    let doc = DocumentObject::new(
        1,
        "Test",
        None,
        "txt",
        "/test.txt",
        "2025-05-23T22:44:00+00:00",
        None,
        None,
        Some(0),
        None,
    );

    let formatted = doc.format_date();
    // Should contain date components
    assert!(!formatted.is_empty());
    assert!(formatted.contains("23") || formatted.contains("May") || formatted.contains("2025"));
}

#[test]
fn test_document_object_default_values() {
    // Create a new DocumentObject with default-like values
    let doc = DocumentObject::new(0, "", None, "", "", "", None, None, Some(0), None);

    assert_eq!(doc.id(), 0);
    assert_eq!(doc.title(), "");
    // When author is empty string, display_author returns Some("") for empty or "Unknown" for None
    // The DocumentObject stores empty string as author
    assert!(doc.author().is_some());
    assert_eq!(doc.format(), "");
    assert_eq!(doc.path(), "");
    assert_eq!(doc.file_size_bytes(), 0);
    // Text encoding is stored as empty string when None
    assert!(doc.text_encoding().is_some());
}

#[test]
fn test_document_object_display_path() {
    let doc = DocumentObject::new(
        1,
        "Test",
        None,
        "txt",
        "/home/user/documents/test.txt",
        "2025-05-23T22:44:00+00:00",
        None,
        None,
        Some(0),
        None,
    );

    let display = doc.display_path();
    // Should show just the filename
    assert_eq!(display, "test.txt");
}

#[test]
fn test_document_object_from_database_document() {
    let db_doc = Document {
        id: 1,
        title: "Database Document".to_string(),
        author: Some("DB Author".to_string()),
        format: "md".to_string(),
        path: "/path/to/doc.md".to_string(),
        date_added: "2025-05-23T22:44:00+00:00".to_string(),
        last_opened: Some("2025-05-24T10:30:00+00:00".to_string()),
        cover_path: None,
        file_size_bytes: Some(2048),
        text_encoding: Some("UTF-8".to_string()),
    };

    let doc_obj = DocumentObject::from(&db_doc);

    assert_eq!(doc_obj.id(), 1);
    assert_eq!(doc_obj.title(), "Database Document");
    assert_eq!(doc_obj.author(), Some("DB Author".to_string()));
    assert_eq!(doc_obj.format(), "md");
    assert_eq!(doc_obj.file_size_bytes(), 2048);
}
