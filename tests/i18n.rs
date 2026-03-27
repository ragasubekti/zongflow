use zongflow::i18n::{get_current_locale, map_system_locale, set_current_locale, translate_format};

#[test]
fn test_map_system_locale_chinese() {
    assert_eq!(map_system_locale("zh_CN"), "zh_CN");
    assert_eq!(map_system_locale("zh_CN.UTF-8"), "zh_CN");
    assert_eq!(map_system_locale("zh_TW"), "zh_CN");
    assert_eq!(map_system_locale("zh_HK.UTF-8"), "zh_CN");
}

#[test]
fn test_map_system_locale_japanese() {
    assert_eq!(map_system_locale("ja_JP"), "ja_JP");
    assert_eq!(map_system_locale("ja_JP.UTF-8"), "ja_JP");
}

#[test]
fn test_map_system_locale_english() {
    assert_eq!(map_system_locale("en_US"), "en_US");
    assert_eq!(map_system_locale("en_GB.UTF-8"), "en_US");
    assert_eq!(map_system_locale("de_DE"), "en_US");
}

#[test]
fn test_map_system_locale_colon_separated() {
    assert_eq!(map_system_locale("zh_CN:en"), "zh_CN");
    assert_eq!(map_system_locale("ja_JP:en_US:zh_CN"), "ja_JP");
    assert_eq!(map_system_locale("en_US:zh_CN"), "en_US");
}

#[test]
fn test_set_current_locale_is_idempotent() {
    set_current_locale("zh_CN");
    let locale1 = get_current_locale();
    set_current_locale("zh_CN");
    let locale2 = get_current_locale();
    assert_eq!(locale1, locale2);
}

#[test]
fn test_translate_format() {
    // This test just ensures no panics occur; actual translation depends on .mo files
    let result = translate_format("Hello { $name }", &[("name", "World")]);
    // If the key isn't translated, it returns the key as-is
    // The placeholder should still be replaced
    assert!(result.contains("World"));
}
