mod imp;

use adw::prelude::*;
use adw::subclass::prelude::ObjectSubclassIsExt;
use gio::Menu;
use gio::SimpleAction;
use glib::object::ObjectExt;
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
        window.setup_view_toggle();
        window.setup_import_actions();
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
            "edit-redo-symbolic",
        );

        let translate = TranslateWidget::new();
        if let Some(db) = db.clone() {
            translate.set_db(db);
        }
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
            let Some(window) = window_weak.upgrade() else {
                return;
            };

            let settings = SettingsWidget::new();
            if let Some(db) = db.clone() {
                settings.set_db(db);
            }
            settings.set_window(&window);

            let dialog = adw::PreferencesDialog::new();
            dialog.set_title(&i18n::translate("SETTINGS"));
            dialog.add(&settings);
            dialog.present(Some(&window));
        });
    }

    fn setup_view_toggle(&self) {
        let stack = self.imp().stack.get();
        let toggle_group = self.imp().view_toggle_group.get();

        // Get library widget to access its view stack
        let Some(library_page) = stack.child_by_name("library") else {
            return;
        };
        let Ok(library) = library_page.downcast::<LibraryWidget>() else {
            return;
        };
        let library_view_stack = library.imp().view_stack.get();

        // Connect toggle group to switch library view
        let view_stack = library_view_stack.clone();
        toggle_group.connect_active_name_notify(move |group| {
            if let Some(name) = group.active_name() {
                view_stack.set_visible_child_name(&name);
            }
        });

        // Show/hide toggle group based on active page
        let toggle_group_weak = toggle_group.downgrade();
        stack.connect_visible_child_name_notify(move |stack| {
            let Some(toggle_group) = toggle_group_weak.upgrade() else {
                return;
            };
            let is_library = stack
                .visible_child_name()
                .map(|name| name.as_str() == "library")
                .unwrap_or(false);
            toggle_group.set_visible(is_library);
        });

        // Set initial state
        let is_library = stack
            .visible_child_name()
            .map(|name| name.as_str() == "library")
            .unwrap_or(false);
        toggle_group.set_visible(is_library);
        toggle_group.set_active_name(Some("list"));
    }

    fn setup_import_actions(&self) {
        let stack = self.imp().stack.get();
        let import_button = self.imp().import_button.get();

        // Get application action map
        let app = self.application().expect("Window must have application");
        let app = app
            .downcast_ref::<adw::Application>()
            .expect("Application must be adw::Application");

        // Create menu model for dropdown
        let menu = Menu::new();
        menu.append(
            Some(&i18n::translate("IMPORT_FILE")),
            Some("app.import-file"),
        );
        menu.append(
            Some(&i18n::translate("IMPORT_DIRECTORY")),
            Some("app.import-directory"),
        );
        import_button.set_menu_model(Some(&menu));

        // Get convert widget
        let Some(convert_page) = stack.child_by_name("convert") else {
            return;
        };
        let Ok(convert) = convert_page.downcast::<ConvertWidget>() else {
            return;
        };

        // Create actions
        let action_import_file = SimpleAction::new("import-file", None);
        let convert_clone = convert.clone();
        action_import_file.connect_activate(move |_, _| {
            convert_clone.imp().on_file_button_clicked();
        });
        app.add_action(&action_import_file);

        let action_import_directory = SimpleAction::new("import-directory", None);
        let convert_clone2 = convert.clone();
        action_import_directory.connect_activate(move |_, _| {
            convert_clone2.imp().on_directory_button_clicked();
        });
        app.add_action(&action_import_directory);

        // Set visibility based on active page
        let import_button_weak = import_button.downgrade();
        stack.connect_visible_child_name_notify(move |stack| {
            let Some(button) = import_button_weak.upgrade() else {
                return;
            };
            let is_convert = stack
                .visible_child_name()
                .map(|name| name.as_str() == "convert")
                .unwrap_or(false);
            button.set_visible(is_convert);
        });

        // Set initial visibility
        let is_convert = stack
            .visible_child_name()
            .map(|name| name.as_str() == "convert")
            .unwrap_or(false);
        import_button.set_visible(is_convert);
    }

    pub fn update_ui_strings(&self) {
        tracing::debug!("Window::update_ui_strings called");
        self.imp().update_ui_strings();

        // Update child widgets
        if let Some(page) = self.imp().stack.child_by_name("library") {
            if let Ok(library) = page.downcast::<LibraryWidget>() {
                tracing::debug!("Updating library widget");
                library.imp().update_ui_strings();
            }
        }

        if let Some(page) = self.imp().stack.child_by_name("convert") {
            if let Ok(convert) = page.downcast::<ConvertWidget>() {
                tracing::debug!("Updating convert widget");
                convert.imp().update_ui_strings();
            }
        }

        if let Some(page) = self.imp().stack.child_by_name("translate") {
            if let Ok(translate) = page.downcast::<TranslateWidget>() {
                tracing::debug!("Updating translate widget");
                translate.imp().update_ui_strings();
            }
        }
    }
}
