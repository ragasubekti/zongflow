use crate::config;
use gettextrs::{
    bind_textdomain_codeset, bindtextdomain, gettext, setlocale, textdomain, LocaleCategory,
};
use std::cell::RefCell;
use std::path::PathBuf;
#[cfg(feature = "debug")]
use std::sync::atomic::{AtomicU32, Ordering};
use tracing;

#[cfg(feature = "debug")]
// Counter for debug logging (thread-safe, no unsafe)
static TRANSLATE_COUNT: AtomicU32 = AtomicU32::new(0);

// Thread-local storage for current locale
thread_local! {
    static CURRENT_LANG: RefCell<String> = RefCell::new("en_US".to_string());
}

/// Initialize the i18n system
pub fn init() {
    setlocale(LocaleCategory::LcAll, "");

    let system_locale = std::env::var("LANGUAGE")
        .or_else(|_| std::env::var("LANG"))
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .unwrap_or_else(|_| "en_US".to_string());

    let lang_id = map_system_locale(&system_locale);
    set_locale_internal(&lang_id);
}

/// Map system locale to our supported locales
pub fn map_system_locale(system_locale: &str) -> String {
    // LANGUAGE environment variable may contain colon-separated list, e.g., "zh_CN:en"
    let first_locale = system_locale.split(':').next().unwrap_or("en_US");
    // Strip encoding part (e.g., "zh_CN.UTF-8" -> "zh_CN")
    let locale_without_encoding = first_locale.split('.').next().unwrap_or(first_locale);
    let locale_lower = locale_without_encoding.to_lowercase();

    if locale_lower.starts_with("zh") {
        "zh_CN".to_string()
    } else if locale_lower.starts_with("ja") {
        "ja_JP".to_string()
    } else {
        "en_US".to_string()
    }
}

/// Get the locale directory path
fn get_locale_dir() -> PathBuf {
    let configured = PathBuf::from(config::localedir());
    if configured.exists() {
        return configured;
    }

    let exe_path = std::env::current_exe().unwrap_or_default();
    let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new(""));

    let possible_paths = [
        exe_dir.join("share/locale"),
        exe_dir.join("../share/locale"),
        exe_dir.join("../../locales"), // when binary is in target/debug/
        PathBuf::from("locales"),
        PathBuf::from("/usr/share/locale"),
    ];

    for path in possible_paths {
        if path.exists() {
            return path;
        }
    }

    configured
}

/// Set locale internally
fn set_locale_internal(lang_id: &str) {
    CURRENT_LANG.with(|current| {
        *current.borrow_mut() = lang_id.to_string();
    });

    std::env::set_var("LANGUAGE", lang_id);

    let locale_dir = get_locale_dir();
    let gettext_package = config::gettext_package();

    // Try to set the locale for the target language
    if lang_id.starts_with("zh") {
        let _ = setlocale(LocaleCategory::LcAll, "zh_CN.UTF-8")
            .or_else(|| setlocale(LocaleCategory::LcAll, "zh_CN"));
    } else if lang_id.starts_with("ja") {
        let _ = setlocale(LocaleCategory::LcAll, "ja_JP.UTF-8")
            .or_else(|| setlocale(LocaleCategory::LcAll, "ja_JP"));
    }

    if let Err(e) = bindtextdomain(gettext_package, &locale_dir) {
        tracing::warn!("Failed to bind text domain: {}", e);
    }
    if let Err(e) = bind_textdomain_codeset(gettext_package, "UTF-8") {
        tracing::warn!("Failed to bind text domain codeset: {}", e);
    }
    if let Err(e) = textdomain(gettext_package) {
        tracing::warn!("Failed to set text domain: {}", e);
    }
}

/// Get current locale
pub fn get_current_locale() -> String {
    CURRENT_LANG.with(|current| current.borrow().clone())
}

/// Set locale
pub fn set_current_locale(locale: &str) {
    let lang_id = map_system_locale(locale);
    set_locale_internal(&lang_id);
}

/// Translation helper function
pub fn translate(key: &str) -> String {
    let result = gettext(key).to_string();
    #[cfg(feature = "debug")]
    {
        let count = TRANSLATE_COUNT.fetch_add(1, Ordering::Relaxed);
        if count < 20 {
            tracing::debug!("i18n: translate('{}') = '{}'", key, result);
        }
    }
    result
}

/// Format a string with parameters (for gettext parameterized strings)
/// Handles strings like "Selected: { $count }"
pub fn translate_format(key: &str, params: &[(&str, &str)]) -> String {
    let mut result = translate(key);

    for (param_key, param_value) in params {
        let placeholder = format!("{{ ${} }}", param_key);
        result = result.replace(&placeholder, param_value);
    }

    result
}
