mod imp;

use adw::prelude::*;
use adw::subclass::prelude::ObjectSubclassIsExt;
use glib::Object;
use gtk::glib;

use crate::convert::ConvertWidget;
use crate::database::Database;
use crate::i18n;
use crate::library::LibraryWidget;
use crate::settings::SettingsWidget;
use crate::translate::TranslateWidget;
use tracing;

glib::wrapper! {
    pub struct ZongflowWindow(ObjectSubclass<imp::ZongflowWindow>)
        @extends adw::Window, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl ZongflowWindow {
    pub fn new(app: &adw::Application) -> Self {
        let window: ZongflowWindow = Object::builder().property("application", app).build();

        // Initialize database
        let db = match Database::new() {
            Ok(db) => Some(db),
            Err(e) => {
                tracing::error!("Failed to initialize database: {}", e);
                None
            }
        };
        *window.imp().db.borrow_mut() = db;

        window.setup_pages();
        window.setup_settings_button();
        window.update_ui_strings();
        window
    }

    fn setup_pages(&self) {
        let stack: adw::ViewStack = self.imp().stack.get();
        let db = self.imp().db.borrow().clone();

        let library = LibraryWidget::new();
        if let Some(db) = db.clone() {
            library.set_db(db);
        }
        library.set_visible(true);
        stack.add_titled_with_icon(
            &library,
            Some("library"),
            &i18n::translate("LIBRARY"),
            "folder-documents-symbolic",
        );

        let convert = ConvertWidget::new();
        if let Some(db) = db.clone() {
            convert.set_db(db);
        }
        convert.set_visible(true);
        stack.add_titled_with_icon(
            &convert,
            Some("convert"),
            &i18n::translate("CONVERT"),
            "shuffle-symbolic",
        );

        let translate = TranslateWidget::new();
        translate.set_visible(true);
        stack.add_titled_with_icon(
            &translate,
            Some("translate"),
            &i18n::translate("TRANSLATE"),
            "preferences-desktop-locale-symbolic",
        );

        stack.set_visible_child_name("library");
    }

    fn setup_settings_button(&self) {
        let db = self.imp().db.borrow().clone();
        let button = self.imp().settings_button.get();
        let window_weak = self.downgrade();

        button.connect_clicked(move |_| {
            let Some(window) = window_weak.upgrade() else { return };

            let settings = SettingsWidget::new();
            if let Some(db) = db.clone() {
                settings.set_db(db);
            }

            let dialog = adw::PreferencesDialog::new();
            dialog.set_title(&i18n::translate("SETTINGS"));
            dialog.add(&settings);
            dialog.present(Some(&window));
        });
    }

    pub fn update_ui_strings(&self) {
        self.imp().update_ui_strings();

        // Update child widgets
        if let Some(page) = self.imp().stack.child_by_name("library") {
            if let Ok(library) = page.downcast::<LibraryWidget>() {
                library.imp().update_ui_strings();
            }
        }

        if let Some(page) = self.imp().stack.child_by_name("convert") {
            if let Ok(convert) = page.downcast::<ConvertWidget>() {
                convert.imp().update_ui_strings();
            }
        }

        if let Some(page) = self.imp().stack.child_by_name("translate") {
            if let Ok(translate) = page.downcast::<TranslateWidget>() {
                translate.imp().update_ui_strings();
            }
        }
    }
}
