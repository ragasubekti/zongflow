use crate::core::DocumentScanner;
use crate::database::Database;
use crate::i18n;
use dirs::home_dir;
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::cell::RefCell;
use tracing;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/github/zongflow/convert.ui")]
pub struct ConvertWidget {
    #[template_child]
    pub title_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub subtitle_label: TemplateChild<gtk::Label>,
    #[template_child]
    pub file_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub directory_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub selection_label: TemplateChild<gtk::Label>,
    pub db: RefCell<Option<Database>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ConvertWidget {
    const NAME: &'static str = "ConvertWidget";
    type Type = super::ConvertWidget;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[gtk::template_callbacks]
impl ConvertWidget {
    #[template_callback]
    fn on_file_button_clicked(&self) {
        let Some(window) = self
            .obj()
            .root()
            .and_then(|w| w.downcast::<gtk::Window>().ok())
        else {
            self.selection_label
                .set_text(&i18n::translate("NO_PARENT_WINDOW"));
            return;
        };
        let file_chooser = gtk::FileChooserNative::new(
            Some(&i18n::translate("SELECT_FILE")),
            Some(&window),
            gtk::FileChooserAction::Open,
            Some(&i18n::translate("OPEN")),
            Some(&i18n::translate("CANCEL")),
        );

        // Add filters
        let txt_filter = gtk::FileFilter::new();
        txt_filter.set_name(Some(&i18n::translate("TEXT_FILES")));
        txt_filter.add_mime_type("text/plain");
        file_chooser.add_filter(&txt_filter);

        let epub_filter = gtk::FileFilter::new();
        epub_filter.set_name(Some(&i18n::translate("EPUB_FILES")));
        epub_filter.add_mime_type("application/epub+zip");
        file_chooser.add_filter(&epub_filter);

        let md_filter = gtk::FileFilter::new();
        md_filter.set_name(Some(&i18n::translate("MARKDOWN_FILES")));
        md_filter.add_pattern("*.md");
        md_filter.add_pattern("*.markdown");
        file_chooser.add_filter(&md_filter);

        let label = self.selection_label.clone();
        let window = window.clone();
        let weak_self = self.obj().downgrade();
        file_chooser.connect_response(move |chooser, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(file) = chooser.file() {
                    if let Some(path) = file.path() {
                        // Upgrade weak reference
                        if let Some(self_ref) = weak_self.upgrade() {
                            if !self_ref.imp().is_path_allowed(&path) {
                                self_ref
                                    .imp()
                                    .show_warning(&window, &i18n::translate("PATH_NOT_ALLOWED"));
                                chooser.destroy();
                                return;
                            }
                        }
                        let path_str = path.to_string_lossy();
                        label.set_text(&i18n::translate_format(
                            "SELECTED_FOLDER",
                            &[("path", &path_str)],
                        ));
                    }
                }
            }
            chooser.destroy();
        });
        file_chooser.show();
    }

    #[template_callback]
    fn on_directory_button_clicked(&self) {
        let Some(window) = self
            .obj()
            .root()
            .and_then(|w| w.downcast::<gtk::Window>().ok())
        else {
            tracing::warn!("No parent window found for directory chooser");
            self.selection_label
                .set_text(&i18n::translate("NO_PARENT_WINDOW"));
            return;
        };
        let folder_chooser = gtk::FileChooserNative::new(
            Some(&i18n::translate("SELECT_DIRECTORY")),
            Some(&window),
            gtk::FileChooserAction::SelectFolder,
            Some(&i18n::translate("OPEN")),
            Some(&i18n::translate("CANCEL")),
        );
        let label = self.selection_label.clone();
        let db = self.db.clone();
        let window = window.clone();
        let weak_self = self.obj().downgrade();
        folder_chooser.connect_response(move |chooser, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(folder) = chooser.file() {
                    if let Some(path) = folder.path() {
                        // Upgrade weak reference
                        if let Some(self_ref) = weak_self.upgrade() {
                            if !self_ref.imp().is_path_allowed(&path) {
                                tracing::warn!(path = ?path, "Path not allowed for scanning");
                                self_ref
                                    .imp()
                                    .show_warning(&window, &i18n::translate("PATH_NOT_ALLOWED"));
                                chooser.destroy();
                                return;
                            }
                        }
                        let path_str = path.to_string_lossy();
                        tracing::info!(path = ?path, "User selected directory for scanning");
                        label.set_text(&i18n::translate_format(
                            "SELECTED_FOLDER",
                            &[("path", &path_str)],
                        ));
                        // Scan directory with DocumentScanner and add to database
                        if let Some(db) = db.borrow().as_ref() {
                            match DocumentScanner::scan_directory(&path, db) {
                                Ok(docs) => {
                                    tracing::info!(path = ?path, count = docs.len(), "Directory scan completed");
                                    label.set_text(&i18n::translate_format(
                                        "FOUND_DOCUMENTS",
                                        &[("count", &docs.len().to_string())],
                                    ));
                                }
                                Err(e) => {
                                    tracing::error!(path = ?path, error = %e, "Failed to scan directory");
                                    label.set_text(&i18n::translate_format(
                                        "ERROR_SCANNING",
                                        &[("error", &e.to_string())],
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            chooser.destroy();
        });
        folder_chooser.show();
    }

    pub fn update_ui_strings(&self) {
        self.title_label.set_label(&i18n::translate("CONVERT"));
        self.subtitle_label
            .set_label(&i18n::translate("SELECT_FILES_OR_DIRECTORIES"));
        self.file_button.set_label(&i18n::translate("PICK_FILE"));
        self.directory_button
            .set_label(&i18n::translate("PICK_DIRECTORY"));
        self.selection_label
            .set_label(&i18n::translate("NO_SELECTION"));
    }

    fn is_path_allowed(&self, path: &std::path::Path) -> bool {
        if let Some(home) = home_dir() {
            path.starts_with(home)
        } else {
            true // cannot determine home, allow
        }
    }

    fn show_warning(&self, parent: &gtk::Window, message: &str) {
        let dialog = gtk::MessageDialog::new(
            Some(parent),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Warning,
            gtk::ButtonsType::Ok,
            message,
        );
        dialog.connect_response(|dialog, _| dialog.close());
        dialog.show();
    }
}

impl ObjectImpl for ConvertWidget {}
impl WidgetImpl for ConvertWidget {}
impl BoxImpl for ConvertWidget {}
