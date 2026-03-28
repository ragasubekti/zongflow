use crate::database::Database;
use crate::i18n;
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use gtk::CompositeTemplate;
use std::cell::RefCell;
use tracing;

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
    pub view_toggle_group: TemplateChild<adw::ToggleGroup>,
    #[template_child]
    pub import_button: TemplateChild<adw::SplitButton>,
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
        tracing::debug!("ZongflowWindow::update_ui_strings called");
        let stack = &self.stack;

        if let Some(library_page) = stack.child_by_name("library") {
            let page = stack.page(&library_page);
            let title = i18n::translate("LIBRARY");
            tracing::debug!("Setting library title to {}", title);
            page.set_title(Some(&title));
        }

        if let Some(convert_page) = stack.child_by_name("convert") {
            let page = stack.page(&convert_page);
            let title = i18n::translate("CONVERT");
            tracing::debug!("Setting convert title to {}", title);
            page.set_title(Some(&title));
        }

        if let Some(translate_page) = stack.child_by_name("translate") {
            let page = stack.page(&translate_page);
            let title = i18n::translate("TRANSLATE");
            tracing::debug!("Setting translate title to {}", title);
            page.set_title(Some(&title));
        }
    }
}
