use crate::i18n;
use glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/github/zongflow/translate.ui")]
pub struct TranslateWidget {
    #[template_child]
    pub title_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub subtitle_label: TemplateChild<gtk::Label>,
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
    }
}

impl ObjectImpl for TranslateWidget {}
impl WidgetImpl for TranslateWidget {}
impl BoxImpl for TranslateWidget {}
