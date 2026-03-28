mod imp;

use crate::database::Database;
use glib::Object;
use gtk::glib;
use gtk::subclass::prelude::ObjectSubclassIsExt;

glib::wrapper! {
    pub struct SettingsWidget(ObjectSubclass<imp::SettingsWidget>)
        @extends adw::PreferencesPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl SettingsWidget {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn set_db(&self, db: Database) {
        *self.imp().db.borrow_mut() = Some(db);
        self.imp().init_ui();
    }

    pub fn save_pending(&self) {
        self.imp().save_pending();
    }
}

impl Default for SettingsWidget {
    fn default() -> Self {
        Self::new()
    }
}
