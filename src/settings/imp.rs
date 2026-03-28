use crate::core::SettingsManager;
use crate::database::Database;
use crate::i18n;
use crate::window::ZongflowWindow;
use adw::prelude::*;
use adw::subclass::prelude::PreferencesPageImpl;
use glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::cell::Cell;
use std::cell::RefCell;
use tracing;

const LANGUAGES: [&str; 3] = ["zh_CN", "en_US", "ja_JP"];

const TITLE_NAMES: [(&str, &str); 16] = [
    ("General", "general"),
    ("Maintenance", "maintenance"),
    ("Language", "general-language"),
    ("Dark Mode", "general-dark-mode"),
    ("Output Folder", "general-output-folder"),
    ("Clear Cache", "maintenance-clear-cache"),
    ("Clear Database", "maintenance-clear-database"),
    ("Reset Settings", "maintenance-reset-settings"),
    ("About", "maintenance-about"),
    ("Application language", "general-language-subtitle"),
    ("Use dark theme", "general-dark-mode-subtitle"),
    (
        "Where converted files are saved",
        "general-output-folder-subtitle",
    ),
    ("Delete temporary files", "maintenance-clear-cache-subtitle"),
    (
        "Remove all library data",
        "maintenance-clear-database-subtitle",
    ),
    ("Restore defaults", "maintenance-reset-settings-subtitle"),
    ("Application information", "maintenance-about-subtitle"),
];

const TITLE_TRANSLATIONS: [(&str, &str); 9] = [
    ("general", "GENERAL"),
    ("maintenance", "MAINTENANCE"),
    ("general-language", "LANGUAGE"),
    ("general-dark-mode", "DARK_MODE"),
    ("general-output-folder", "OUTPUT_FOLDER"),
    ("maintenance-clear-cache", "CLEAR_CACHE"),
    ("maintenance-clear-database", "CLEAR_DATABASE"),
    ("maintenance-reset-settings", "RESET_SETTINGS"),
    ("maintenance-about", "ABOUT"),
];

const SUBTITLE_TRANSLATIONS: [(&str, &str); 7] = [
    ("general-language", "APPLICATION_LANGUAGE"),
    ("general-dark-mode", "USE_DARK_THEME"),
    ("general-output-folder", "WHERE_CONVERTED_SAVED"),
    ("maintenance-clear-cache", "DELETE_TEMPORARY_FILES"),
    ("maintenance-clear-database", "REMOVE_ALL_LIBRARY_DATA"),
    ("maintenance-reset-settings", "RESTORE_DEFAULTS"),
    ("maintenance-about", "APPLICATION_INFORMATION"),
];

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/github/zongflow/settings.ui")]
pub struct SettingsWidget {
    #[template_child]
    pub language_dropdown: TemplateChild<gtk::DropDown>,
    #[template_child]
    pub dark_mode_switch: TemplateChild<gtk::Switch>,
    #[template_child]
    pub output_folder_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub clear_cache_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub clear_database_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub reset_settings_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub about_button: TemplateChild<gtk::Button>,
    pub db: RefCell<Option<Database>>,
    pub window: RefCell<Option<glib::WeakRef<ZongflowWindow>>>,
    refreshing_dropdown: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for SettingsWidget {
    const NAME: &'static str = "SettingsWidget";
    type Type = super::SettingsWidget;
    type ParentType = adw::PreferencesPage;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[gtk::template_callbacks]
impl SettingsWidget {
    pub fn init_ui(&self) {
        let db_borrow = self.db.borrow();
        let Some(db) = db_borrow.as_ref() else { return };
        let mgr = SettingsManager::new(db.clone());

        // Set widget names from tree structure (only on first init)
        self.set_names_from_tree(self.obj().upcast_ref());

        // Language dropdown
        let current = mgr.get_language();
        self.update_language_dropdown(&LANGUAGES);
        let pos = LANGUAGES.iter().position(|&s| s == current).unwrap_or(1) as u32;
        self.refreshing_dropdown.set(true);
        self.language_dropdown.set_selected(pos);
        self.refreshing_dropdown.set(false);

        // Dark mode switch
        self.dark_mode_switch.set_active(mgr.get_dark_mode());

        // Update all UI strings with current language
        self.update_ui_strings();

        // Manually connect output folder button signal as fallback
        let weak_self = self.obj().downgrade();
        self.output_folder_button.connect_clicked(move |_| {
            let Some(this) = weak_self.upgrade() else {
                return;
            };
            this.imp().on_output_folder_button_clicked();
        });

        // Connect language change — defer heavy work to avoid UI freeze
        let weak_self = self.obj().downgrade();
        self.language_dropdown.connect_selected_notify(move |dd| {
            let Some(this) = weak_self.upgrade() else {
                return;
            };
            let imp = this.imp();
            if imp.refreshing_dropdown.get() {
                return;
            }
            let idx = dd.selected();
            let lang = LANGUAGES[idx as usize].to_string();

            let weak = this.downgrade();
            glib::idle_add_local_once(move || {
                let Some(this) = weak.upgrade() else {
                    return;
                };
                let imp = this.imp();
                let db_borrow = imp.db.borrow();
                if let Some(db) = db_borrow.as_ref() {
                    let mut mgr = SettingsManager::new(db.clone());
                    match mgr.set_language(&lang) {
                        Ok(_) => {
                            i18n::set_current_locale(&lang);
                            imp.refresh_language_dropdown();
                            imp.update_ui_strings();
                            imp.notify_parent_window();
                            imp.show_toast(&i18n::translate("SETTINGS_SAVED_MESSAGE"));
                        }
                        Err(e) => {
                            imp.show_toast(&format!("Failed to save language: {}", e));
                        }
                    }
                }
            });
        });

        // Connect dark mode switch — save immediately
        let this = self.obj().clone();
        self.dark_mode_switch.connect_active_notify(move |sw| {
            let imp = this.imp();
            let dark_mode = sw.is_active();

            // Preview dark mode immediately
            let style_manager = adw::StyleManager::default();
            style_manager.set_color_scheme(if dark_mode {
                adw::ColorScheme::ForceDark
            } else {
                adw::ColorScheme::Default
            });

            // Save immediately
            let db_borrow = imp.db.borrow();
            if let Some(db) = db_borrow.as_ref() {
                let mut mgr = SettingsManager::new(db.clone());
                match mgr.set_dark_mode(dark_mode) {
                    Ok(_) => {
                        imp.show_toast(&i18n::translate("SETTINGS_SAVED_MESSAGE"));
                    }
                    Err(e) => {
                        imp.show_toast(&format!("Failed to save dark mode: {}", e));
                    }
                }
            }
        });
    }

    /// Reset all widget names so set_names_from_tree can re-match from fresh titles.
    fn reset_widget_names(&self) {
        self.reset_names_recursive(self.obj().upcast_ref());
    }

    fn reset_names_recursive(&self, widget: &gtk::Widget) {
        if widget.downcast_ref::<adw::PreferencesGroup>().is_some()
            || widget.downcast_ref::<adw::ActionRow>().is_some()
        {
            widget.set_widget_name("");
        }
        let mut child = widget.first_child();
        while let Some(c) = child {
            self.reset_names_recursive(&c);
            child = c.next_sibling();
        }
    }

    /// Set translation keys as widget names by matching initial English titles.
    /// Called once during init_ui before any translation occurs.
    fn set_names_from_tree(&self, widget: &gtk::Widget) {
        let title_names = TITLE_NAMES;

        if let Some(group) = widget.downcast_ref::<adw::PreferencesGroup>() {
            let title = group.title().to_string();
            for (en, name) in &title_names {
                if title == *en {
                    group.set_widget_name(name);
                    break;
                }
            }
        }

        if let Some(row) = widget.downcast_ref::<adw::ActionRow>() {
            let title = row.title().to_string();
            for (en, name) in &title_names {
                if title == *en {
                    row.set_widget_name(name);
                    break;
                }
            }
            if let Some(subtitle) = row.subtitle() {
                let sub = subtitle.to_string();
                for (en, _name) in &title_names {
                    if sub == *en {
                        break;
                    }
                }
            }
            // Also traverse into action row suffix children
            if let Some(child) = row.first_child() {
                self.set_names_from_tree(&child);
            }
        }

        let mut child = widget.first_child();
        while let Some(c) = child {
            self.set_names_from_tree(&c);
            child = c.next_sibling();
        }
    }

    pub fn update_ui_strings(&self) {
        // Update button labels
        self.output_folder_button
            .set_label(&i18n::translate("CHOOSE"));
        self.clear_cache_button.set_label(&i18n::translate("CLEAR"));
        self.clear_database_button
            .set_label(&i18n::translate("CLEAR"));
        self.reset_settings_button
            .set_label(&i18n::translate("RESET"));
        self.about_button.set_label(&i18n::translate("ABOUT"));
        // Update preference group and row titles/subtitles via widget names
        if let Some(page) = self.obj().first_child() {
            self.update_widget_tree(&page);
        }

        self.obj().queue_draw();
    }

    fn update_widget_tree(&self, widget: &gtk::Widget) {
        // Title translation map
        let title_translations = &TITLE_TRANSLATIONS;

        // Subtitle translation map (key prefix -> translation key)
        let subtitle_translations = &SUBTITLE_TRANSLATIONS;

        if let Some(group) = widget.downcast_ref::<adw::PreferencesGroup>() {
            let name = group.widget_name();
            let name_str = name.as_str();
            if !name_str.is_empty() {
                for (key, translation_key) in title_translations {
                    if name_str == *key {
                        group.set_title(&i18n::translate(translation_key));
                        break;
                    }
                }
            }
        }

        if let Some(row) = widget.downcast_ref::<adw::ActionRow>() {
            let name = row.widget_name();
            let name_str = name.as_str();
            if !name_str.is_empty() {
                // Translate title
                for (key, translation_key) in title_translations {
                    if name_str == *key {
                        row.set_title(&i18n::translate(translation_key));
                        break;
                    }
                }

                // Translate subtitle
                for (key_prefix, translation_key) in subtitle_translations {
                    if name_str == *key_prefix {
                        row.set_subtitle(&i18n::translate(translation_key));
                        break;
                    }
                }
            }
        }

        // Recursively update children
        let mut child = widget.first_child();
        while let Some(child_widget) = child {
            self.update_widget_tree(&child_widget);
            child = child_widget.next_sibling();
        }
    }

    fn update_language_dropdown(&self, _languages: &[&str]) {
        let language_names = [
            i18n::translate("LANG_ZH_CN"),
            i18n::translate("LANG_EN_US"),
            i18n::translate("LANG_JA_JP"),
        ];

        // Try to update existing StringList model in place
        if let Some(model) = self.language_dropdown.model() {
            if let Some(string_list) = model.downcast_ref::<gtk::StringList>() {
                for (i, name) in language_names.iter().enumerate() {
                    if (i as u32) < string_list.n_items() {
                        string_list.splice(i as u32, 1, &[name.as_str()]);
                    }
                }
                return;
            }
        }

        // Fallback: create a new StringList
        let language_name_refs: Vec<&str> = language_names.iter().map(|s| s.as_str()).collect();
        let string_list = gtk::StringList::new(&language_name_refs);
        self.language_dropdown.set_model(Some(&string_list));
    }

    fn refresh_language_dropdown(&self) {
        if self.refreshing_dropdown.get() {
            return;
        }
        self.refreshing_dropdown.set(true);
        self.update_language_dropdown(&LANGUAGES);
        self.refreshing_dropdown.set(false);
    }

    fn show_toast(&self, message: &str) {
        let toast = adw::Toast::new(message);
        if let Some(parent) = self.obj().parent() {
            if let Some(dialog) = parent.downcast_ref::<adw::PreferencesDialog>() {
                dialog.add_toast(toast);
            }
        }
    }

    fn notify_parent_window(&self) {
        tracing::debug!("notify_parent_window called");
        let window_ref = self.window.borrow();
        if let Some(weak_window) = window_ref.as_ref() {
            if let Some(window) = weak_window.upgrade() {
                tracing::debug!("Calling ZongflowWindow::update_ui_strings");
                window.update_ui_strings();
            } else {
                tracing::warn!("Window reference is dead");
            }
        } else {
            tracing::warn!("No window reference stored");
        }
    }

    #[template_callback]
    fn on_output_folder_button_clicked(&self) {
        tracing::info!("Output folder button clicked");
        let Some(window) = self
            .obj()
            .root()
            .and_then(|w| w.downcast::<gtk::Window>().ok())
        else {
            return;
        };
        let folder_chooser = gtk::FileChooserNative::new(
            Some(&i18n::translate("SELECT_OUTPUT_FOLDER")),
            Some(&window),
            gtk::FileChooserAction::SelectFolder,
            Some(&i18n::translate("OPEN")),
            Some(&i18n::translate("CANCEL")),
        );
        let this = self.obj().clone();

        folder_chooser.connect_response(move |chooser, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(folder) = chooser.file() {
                    if let Some(path) = folder.path() {
                        let imp = this.imp();

                        // Save immediately
                        let db_borrow = imp.db.borrow();
                        if let Some(db) = db_borrow.as_ref() {
                            let mut mgr = SettingsManager::new(db.clone());
                            match mgr.set_output_folder(&path) {
                                Ok(_) => {
                                    imp.show_toast(&i18n::translate("SETTINGS_SAVED_MESSAGE"));
                                }
                                Err(e) => {
                                    imp.show_toast(&format!("Failed to save output folder: {}", e));
                                }
                            }
                        }
                    }
                }
            }
            chooser.destroy();
        });
        folder_chooser.show();
    }

    #[template_callback]
    fn on_clear_cache_button_clicked(&self) {
        tracing::info!("User clicked clear cache button");
        if let Some(db) = self.db.borrow().as_ref() {
            let mgr = SettingsManager::new(db.clone());
            match mgr.clear_cache() {
                Ok(_) => {
                    tracing::info!("Cache cleared successfully");
                    self.show_toast(&i18n::translate("CACHE_CLEARED_MESSAGE"));
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to clear cache");
                    self.show_toast(&format!("Failed to clear cache: {}", e));
                }
            }
        }
    }

    #[template_callback]
    fn on_clear_database_button_clicked(&self) {
        tracing::warn!("User clicked clear database button");
        if let Some(db) = self.db.borrow().as_ref() {
            let mgr = SettingsManager::new(db.clone());
            match mgr.clear_database() {
                Ok(_) => {
                    tracing::info!("Database cleared successfully");
                    self.show_toast(&i18n::translate("DATABASE_CLEARED_MESSAGE"));
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to clear database");
                    self.show_toast(&format!("Failed to clear database: {}", e));
                }
            }
        }
    }

    #[template_callback]
    fn on_reset_settings_button_clicked(&self) {
        tracing::warn!("User clicked reset settings button");
        if let Some(db) = self.db.borrow().as_ref() {
            let mut mgr = SettingsManager::new(db.clone());
            match mgr.reset_settings() {
                Ok(_) => {
                    tracing::info!("Settings reset successfully");
                    self.reset_widget_names();
                    self.init_ui();
                    self.show_toast(&i18n::translate("SETTINGS_RESET_MESSAGE"));
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to reset settings");
                    self.show_toast(&format!("Failed to reset settings: {}", e));
                }
            }
        }
    }

    #[template_callback]
    fn on_about_button_clicked(&self) {
        let about = adw::AboutDialog::builder()
            .application_name(i18n::translate("APP_TITLE"))
            .application_icon("application-x-executable-symbolic")
            .version(i18n::translate("VERSION"))
            .developers(vec!["Your Name"])
            .copyright(i18n::translate("COPYRIGHT"))
            .build();
        if let Some(window) = self
            .obj()
            .root()
            .and_then(|w| w.downcast::<gtk::Window>().ok())
        {
            about.present(Some(&window));
        }
    }
}

impl ObjectImpl for SettingsWidget {}
impl WidgetImpl for SettingsWidget {}
impl PreferencesPageImpl for SettingsWidget {}
