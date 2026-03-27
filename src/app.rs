use adw::subclass::prelude::*;
use gio::prelude::*;
use glib::subclass::types::ObjectSubclass;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::GtkApplicationImpl;

use crate::window::ZongflowWindow;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ZongflowApplication;

    #[glib::object_subclass]
    impl ObjectSubclass for ZongflowApplication {
        const NAME: &'static str = "ZongflowApplication";
        type Type = super::ZongflowApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for ZongflowApplication {}
    impl ApplicationImpl for ZongflowApplication {
        fn activate(&self) {
            let app = self.obj();
            let window = ZongflowWindow::new(app.upcast_ref());
            window.present();
        }
    }
    impl GtkApplicationImpl for ZongflowApplication {}
    impl AdwApplicationImpl for ZongflowApplication {}
}

glib::wrapper! {
    pub struct ZongflowApplication(ObjectSubclass<imp::ZongflowApplication>)
        @extends adw::Application, gtk::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl ZongflowApplication {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", "com.github.zongflow")
            .property("flags", gio::ApplicationFlags::empty())
            .build()
    }
}

impl Default for ZongflowApplication {
    fn default() -> Self {
        Self::new()
    }
}
