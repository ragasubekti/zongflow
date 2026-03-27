mod imp;

use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct TranslateWidget(ObjectSubclass<imp::TranslateWidget>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl TranslateWidget {
    pub fn new() -> Self {
        Object::builder().build()
    }
}

impl Default for TranslateWidget {
    fn default() -> Self {
        Self::new()
    }
}
