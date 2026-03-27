//! Application configuration constants
//! These are set at compile time via environment variables from the build script

/// Application ID
pub const APP_ID: Option<&str> = option_env!("APP_ID");

/// Gettext package name
pub const GETTEXT_PACKAGE: Option<&str> = option_env!("GETTEXT_PACKAGE");

/// Locale directory path
pub const LOCALEDIR: Option<&str> = option_env!("LOCALEDIR");

/// Get the gettext package name
pub fn gettext_package() -> &'static str {
    GETTEXT_PACKAGE.unwrap_or("zongflow")
}

/// Get the locale directory path
pub fn localedir() -> &'static str {
    LOCALEDIR.unwrap_or("locales")
}
