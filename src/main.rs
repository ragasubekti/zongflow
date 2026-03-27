use gio::prelude::*;
use glib::ExitCode;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

fn init_tracing() {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("zongflow=warn")),
        )
        .init();
}

fn main() -> ExitCode {
    // Initialize tracing
    init_tracing();

    // Initialize libadwaita
    if let Err(e) = adw::init() {
        tracing::error!("Failed to initialize libadwaita: {}", e);
        return ExitCode::FAILURE;
    }

    // Initialize i18n with system locale first
    zongflow::i18n::init();

    // Try to load settings from TOML (via database for compatibility)
    if let Ok(db) = zongflow::database::Database::new() {
        let mgr = zongflow::core::SettingsManager::new(db);
        let saved_locale = mgr.get_language();
        // Override with saved locale if different from system
        zongflow::i18n::set_current_locale(&saved_locale);
        // Apply dark mode setting
        let dark_mode = mgr.get_dark_mode();
        let style_manager = adw::StyleManager::default();
        style_manager.set_color_scheme(if dark_mode {
            adw::ColorScheme::ForceDark
        } else {
            adw::ColorScheme::Default
        });
    }

    // Register and include resources
    if let Err(e) = gio::resources_register_include!("zongflow.gresource") {
        tracing::error!("Failed to register resources: {}", e);
        return ExitCode::FAILURE;
    }

    // Create and run the application
    let app = zongflow::app::ZongflowApplication::new();
    app.run()
}
