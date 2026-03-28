use std::fs;
use tempfile::tempdir;
use zongflow::core::{DocumentScanner, SettingsManager};
use zongflow::database::Database;
use zongflow::test_utils::TestContext;

#[test]
fn test_document_scanner() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("test.txt");
    fs::write(&file1, "Hello").unwrap();
    let file2 = dir.path().join("readme.md");
    fs::write(&file2, "# Title").unwrap();
    let file3 = dir.path().join("image.jpg"); // should be ignored
    fs::write(&file3, "dummy").unwrap();

    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();
    let docs = DocumentScanner::scan_directory(dir.path(), &db).unwrap();
    assert_eq!(docs.len(), 2);
    assert!(docs
        .iter()
        .all(|d| d.format == "Plain Text" || d.format == "Markdown"));
}

#[test]
fn test_settings_manager() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();
    let settings_path = dir.path().join("settings.toml");
    let mut mgr = SettingsManager::new_with_settings_path(db, settings_path);
    assert_eq!(mgr.get_language(), "en_US");
    mgr.set_language("zh_CN").unwrap();
    assert_eq!(mgr.get_language(), "zh_CN");
    assert_eq!(mgr.get_dark_mode(), false);
    mgr.set_dark_mode(true).unwrap();
    assert_eq!(mgr.get_dark_mode(), true);
    // output folder default
    let out = mgr.get_output_folder();
    assert!(out.is_absolute());
    // reset
    mgr.reset_settings().unwrap();
    assert_eq!(mgr.get_language(), "en_US");
    assert_eq!(mgr.get_dark_mode(), false);
}

#[test]
fn test_scanner_empty_directory() {
    let dir = tempdir().unwrap();
    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();
    let docs = DocumentScanner::scan_directory(dir.path(), &db).unwrap();
    assert_eq!(docs.len(), 0);
}

#[test]
fn test_scanner_ignores_unsupported_extensions() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("image.jpg"), "data").unwrap();
    fs::write(dir.path().join("video.mp4"), "data").unwrap();
    fs::write(dir.path().join("archive.zip"), "data").unwrap();

    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();
    let docs = DocumentScanner::scan_directory(dir.path(), &db).unwrap();
    assert_eq!(docs.len(), 0);
}

#[test]
fn test_scanner_detects_epub() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("book.epub"), "epub data").unwrap();

    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();
    let docs = DocumentScanner::scan_directory(dir.path(), &db).unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].format, "EPUB");
    assert_eq!(docs[0].title, "book");
}

#[test]
fn test_scanner_skips_already_inserted() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("doc.txt"), "content").unwrap();

    let db = Database::new_with_path(dir.path().join("test.db")).unwrap();
    let docs1 = DocumentScanner::scan_directory(dir.path(), &db).unwrap();
    assert_eq!(docs1.len(), 1);

    // Scan again — should find existing doc, not insert duplicate
    let docs2 = DocumentScanner::scan_directory(dir.path(), &db).unwrap();
    assert_eq!(docs2.len(), 1);
    assert_eq!(docs1[0].id, docs2[0].id);
}

#[test]
fn test_with_test_context() {
    let ctx = TestContext::new();
    let docs = DocumentScanner::scan_directory(ctx.db_dir.path(), &ctx.db).unwrap();
    assert_eq!(docs.len(), 0);
}
