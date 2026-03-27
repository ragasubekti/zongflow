mod imp;

use crate::database::Database;
use glib::Object;
use gtk::glib;
use gtk::subclass::prelude::ObjectSubclassIsExt;

glib::wrapper! {
    pub struct ConvertWidget(ObjectSubclass<imp::ConvertWidget>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl ConvertWidget {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn set_db(&self, db: Database) {
        *self.imp().db.borrow_mut() = Some(db);
    }
}

impl Default for ConvertWidget {
    fn default() -> Self {
        Self::new()
    }
}
