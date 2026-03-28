use crate::database::Database;
use crate::i18n;
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use gtk::CompositeTemplate;
use std::cell::RefCell;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/github/zongflow/window.ui")]
pub struct ZongflowWindow {
    #[template_child]
    pub stack: TemplateChild<adw::ViewStack>,
    #[template_child]
    pub header_bar: TemplateChild<adw::HeaderBar>,
    #[template_child]
    pub switcher_bar: TemplateChild<adw::ViewSwitcherBar>,
    #[template_child]
    pub settings_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub view_switcher_title: TemplateChild<adw::ViewSwitcherTitle>,
    pub db: RefCell<Option<Database>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ZongflowWindow {
    const NAME: &'static str = "ZongflowWindow";
    type Type = super::ZongflowWindow;
    type ParentType = adw::Window;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for ZongflowWindow {}
impl WidgetImpl for ZongflowWindow {}
impl WindowImpl for ZongflowWindow {}
impl AdwWindowImpl for ZongflowWindow {}

impl ZongflowWindow {
    pub fn update_ui_strings(&self) {
        let stack = &self.stack;

        if let Some(library_page) = stack.child_by_name("library") {
            let page = stack.page(&library_page);
            page.set_title(Some(&i18n::translate("LIBRARY")));
        }

        if let Some(convert_page) = stack.child_by_name("convert") {
            let page = stack.page(&convert_page);
            page.set_title(Some(&i18n::translate("CONVERT")));
        }

        if let Some(translate_page) = stack.child_by_name("translate") {
            let page = stack.page(&translate_page);
            page.set_title(Some(&i18n::translate("TRANSLATE")));
        }
    }
}
