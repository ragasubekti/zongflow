use crate::database::Database;
use crate::i18n;
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use tracing;

use std::cell::RefCell;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/github/zongflow/library.ui")]
pub struct LibraryWidget {
    #[template_child]
    pub title_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub list_view: TemplateChild<gtk::ListBox>,
    #[template_child]
    pub grid_view: TemplateChild<gtk::IconView>,
    #[template_child]
    pub view_stack: TemplateChild<gtk::Stack>,
    pub db: RefCell<Option<Database>>,
}

#[glib::object_subclass]
impl ObjectSubclass for LibraryWidget {
    const NAME: &'static str = "LibraryWidget";
    type Type = super::LibraryWidget;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl LibraryWidget {
    pub fn load_documents(&self) {
        let db_borrow = self.db.borrow();
        let Some(db) = db_borrow.as_ref() else {
            return;
        };
        let docs = match db.list_documents() {
            Ok(docs) => docs,
            Err(e) => {
                tracing::error!("Failed to list documents: {}", e);
                return;
            }
        };

        // Clear existing rows
        let list_box = self.list_view.get();
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }

        // Add a row for each document
        for doc in docs {
            let label = gtk::Label::new(Some(&doc.title));
            label.set_halign(gtk::Align::Start);
            let row = gtk::ListBoxRow::new();
            row.set_child(Some(&label));
            list_box.append(&row);
        }
    }

    pub fn update_ui_strings(&self) {
        self.title_label.set_label(&i18n::translate("LIBRARY"));
    }
}

impl ObjectImpl for LibraryWidget {}
impl WidgetImpl for LibraryWidget {}
impl BoxImpl for LibraryWidget {}
