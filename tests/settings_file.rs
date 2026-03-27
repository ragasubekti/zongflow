use std::fs;
use tempfile::tempdir;
use zongflow::core::Settings;

#[test]
fn test_load_missing_file_returns_default() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("nonexistent.toml");
    let settings = Settings::load(&path).unwrap();
    assert_eq!(settings, Settings::default());
}

#[test]
fn test_save_and_load_round_trip() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("settings.toml");

    let mut settings = Settings::default();
    settings.language = "zh_CN".to_string();
    settings.dark_mode = true;

    settings.save(&path).unwrap();
    let loaded = Settings::load(&path).unwrap();
    assert_eq!(settings, loaded);
}

#[test]
fn test_load_partial_toml_uses_defaults() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("partial.toml");
    fs::write(&path, r#"language = "ja_JP""#).unwrap();

    let settings = Settings::load(&path).unwrap();
    assert_eq!(settings.language, "ja_JP");
    assert_eq!(settings.dark_mode, false); // default
}

#[test]
fn test_load_malformed_toml_returns_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("bad.toml");
    fs::write(&path, "this is not valid toml {{{").unwrap();

    let result = Settings::load(&path);
    assert!(result.is_err());
}
