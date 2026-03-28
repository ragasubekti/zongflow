mod imp;

use crate::database::Database;
use glib::Object;
use gtk::glib;
use gtk::subclass::prelude::ObjectSubclassIsExt;

glib::wrapper! {
    pub struct TranslateWidget(ObjectSubclass<imp::TranslateWidget>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl TranslateWidget {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn set_db(&self, db: Database) {
        *self.imp().db.borrow_mut() = Some(db);
        self.imp().load_documents();
    }
}

impl Default for TranslateWidget {
    fn default() -> Self {
        Self::new()
    }
}
