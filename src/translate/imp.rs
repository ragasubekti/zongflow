use crate::database::Database;
use crate::i18n;
use glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::cell::RefCell;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/github/zongflow/translate.ui")]
pub struct TranslateWidget {
    #[template_child]
    pub title_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub subtitle_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub view_stack: TemplateChild<adw::ViewStack>,
    #[template_child]
    pub status_page: TemplateChild<adw::StatusPage>,
    pub db: RefCell<Option<Database>>,
}

#[glib::object_subclass]
impl ObjectSubclass for TranslateWidget {
    const NAME: &'static str = "TranslateWidget";
    type Type = super::TranslateWidget;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl TranslateWidget {
    pub fn update_ui_strings(&self) {
        self.title_label.set_label(&i18n::translate("TRANSLATE"));
        self.subtitle_label
            .set_label(&i18n::translate("TRANSLATION_COMING_SOON"));
        self.status_page.set_title(&i18n::translate("NO_DOCUMENTS"));
        self.status_page
            .set_description(Some(&i18n::translate("ADD_DOCUMENTS_TO_TRANSLATE")));
    }

    pub fn load_documents(&self) {
        // For now, always show "no files" page
        // Later, when translation documents are implemented, load from database
        self.view_stack.set_visible_child_name("no_files");
    }
}

impl ObjectImpl for TranslateWidget {
    fn constructed(&self) {
        self.parent_constructed();
        // Initially show "no files" page
        self.view_stack.set_visible_child_name("no_files");
    }
}
impl WidgetImpl for TranslateWidget {}
impl BoxImpl for TranslateWidget {}
