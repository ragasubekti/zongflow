mod imp;

use crate::database::Database;
use glib::Object;
use gtk::glib;
use gtk::subclass::prelude::ObjectSubclassIsExt;

glib::wrapper! {
    pub struct LibraryWidget(ObjectSubclass<imp::LibraryWidget>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl LibraryWidget {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn set_db(&self, db: Database) {
        *self.imp().db.borrow_mut() = Some(db);
        self.imp().load_documents();
    }
}

impl Default for LibraryWidget {
    fn default() -> Self {
        Self::new()
    }
}
