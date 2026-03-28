use crate::core::DocumentScanner;
use crate::database::{Database, Document};
use crate::i18n;
use adw::prelude::ExpanderRowExt;
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::cell::RefCell;
use std::rc::Rc;
use tracing;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/github/zongflow/convert.ui")]
pub struct ConvertWidget {
    #[template_child]
    pub output_toggle: TemplateChild<adw::ToggleGroup>,
    #[template_child]
    pub convert_button: TemplateChild<gtk::Button>,

    #[template_child]
    pub list_box: TemplateChild<gtk::ListBox>,
    #[template_child]
    pub import_overlay: TemplateChild<gtk::Box>,
    #[template_child]
    pub status_page: TemplateChild<adw::StatusPage>,
    #[template_child]
    pub content_stack: TemplateChild<adw::ViewStack>,
    pub db: RefCell<Option<Database>>,
    pub selected_format: RefCell<String>,
    pub documents: Rc<RefCell<Vec<Document>>>,
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
    pub fn on_file_button_clicked(&self) {
        tracing::info!("File button clicked");
        // Try to get parent window with fallback
        let window = self
            .obj()
            .root()
            .and_then(|w| w.downcast::<gtk::Window>().ok())
            .or_else(|| {
                self.obj()
                    .ancestor(gtk::Window::static_type())
                    .and_then(|w| w.downcast::<gtk::Window>().ok())
            });

        let Some(window) = window else {
            tracing::warn!("No parent window found for file chooser");
            // Show error dialog even without parent
            self.show_error_dialog(
                &i18n::translate("ERROR"),
                &i18n::translate("NO_PARENT_WINDOW"),
            );
            return;
        };

        let file_chooser = gtk::FileChooserNative::new(
            Some(&i18n::translate("SELECT_FILE")),
            Some(&window),
            gtk::FileChooserAction::Open,
            Some(&i18n::translate("OPEN")),
            Some(&i18n::translate("CANCEL")),
        );

        // Enable multiple file selection
        file_chooser.set_select_multiple(true);

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

        let list_box = self.list_box.clone();
        let documents = self.documents.clone();
        let window = window.clone();
        let weak_self = self.obj().downgrade();
        file_chooser.connect_response(move |chooser, response| {
            if response == gtk::ResponseType::Accept {
                let files = chooser.files();
                let n_items = files.n_items();
                for i in 0..n_items {
                    let Some(file) = files.item(i).and_then(|f| f.downcast::<gio::File>().ok())
                    else {
                        continue;
                    };

                    // Handle non-local URIs
                    let Some(path) = file.path() else {
                        tracing::warn!(uri = %file.uri(), "File has no local path, skipping");
                        if let Some(self_ref) = weak_self.upgrade() {
                            self_ref.imp().show_warning(
                                &window,
                                &format!("{}: {}", i18n::translate("FILE_NOT_LOCAL"), file.uri()),
                            );
                        }
                        continue;
                    };

                    match Self::add_document_from_path(&path, &list_box, &documents) {
                        Ok(_) => {
                            tracing::info!(path = ?path, "File imported to convert list");
                        }
                        Err(e) => {
                            tracing::error!(path = ?path, error = %e, "Failed to import file");
                            if let Some(self_ref) = weak_self.upgrade() {
                                self_ref.imp().show_warning(
                                    &window,
                                    &format!("{}: {}", i18n::translate("IMPORT_FAILED"), e),
                                );
                            }
                        }
                    }

                    if let Some(self_ref) = weak_self.upgrade() {
                        self_ref.imp().update_convert_button();
                    }
                }
            }
            chooser.destroy();
        });
        file_chooser.show();
    }

    #[template_callback]
    pub fn on_directory_button_clicked(&self) {
        tracing::info!("Directory button clicked");
        // Try to get parent window with fallback
        let window = self
            .obj()
            .root()
            .and_then(|w| w.downcast::<gtk::Window>().ok())
            .or_else(|| {
                self.obj()
                    .ancestor(gtk::Window::static_type())
                    .and_then(|w| w.downcast::<gtk::Window>().ok())
            });

        let Some(window) = window else {
            tracing::warn!("No parent window found for directory chooser");
            self.show_error_dialog(
                &i18n::translate("ERROR"),
                &i18n::translate("NO_PARENT_WINDOW"),
            );
            return;
        };

        let folder_chooser = gtk::FileChooserNative::new(
            Some(&i18n::translate("SELECT_DIRECTORY")),
            Some(&window),
            gtk::FileChooserAction::SelectFolder,
            Some(&i18n::translate("OPEN")),
            Some(&i18n::translate("CANCEL")),
        );

        let list_box = self.list_box.clone();
        let documents = self.documents.clone();
        let window = window.clone();
        let weak_self = self.obj().downgrade();
        folder_chooser.connect_response(move |chooser, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(folder) = chooser.file() {
                    // Handle non-local URIs
                    let Some(path) = folder.path() else {
                        tracing::warn!(uri = %folder.uri(), "Folder has no local path");
                        if let Some(self_ref) = weak_self.upgrade() {
                            self_ref.imp().show_warning(
                                &window,
                                &format!("{}: {}", i18n::translate("FOLDER_NOT_LOCAL"), folder.uri()),
                            );
                        }
                        chooser.destroy();
                        return;
                    };

                    tracing::info!(path = ?path, "Scanning directory for convert list");
                    match Self::scan_directory_local(&path) {
                        Ok(docs) => {
                            tracing::info!(path = ?path, count = docs.len(), "Directory scan completed");
                            for doc in &docs {
                                let row = Self::create_expander_row(doc);
                                list_box.append(&row);
                            }
                            documents.borrow_mut().extend(docs);
                        }
                        Err(e) => {
                            tracing::error!(path = ?path, error = %e, "Failed to scan directory");
                            if let Some(self_ref) = weak_self.upgrade() {
                                self_ref.imp().show_warning(
                                    &window,
                                    &format!("{}: {}", i18n::translate("SCAN_FAILED"), e),
                                );
                            }
                        }
                    }

                    if let Some(self_ref) = weak_self.upgrade() {
                        self_ref.imp().update_convert_button();
                    }
                }
            }
            chooser.destroy();
        });
        folder_chooser.show();
    }

    fn add_document_from_path(
        path: &std::path::Path,
        list_box: &gtk::ListBox,
        documents: &RefCell<Vec<Document>>,
    ) -> anyhow::Result<()> {
        let path_str = path.to_str().unwrap_or_default();

        let title = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let format = DocumentScanner::normalize_format(&ext);

        let file_size_bytes = std::fs::metadata(path).ok().map(|m| m.len() as i64);
        let text_encoding = match ext.as_str() {
            "txt" | "md" | "markdown" => Some("UTF-8".to_string()),
            _ => None,
        };

        let doc = Document {
            id: -1,
            title,
            author: Some("Unknown".to_string()),
            format,
            path: path_str.to_string(),
            date_added: chrono::Utc::now().to_rfc3339(),
            last_opened: None,
            cover_path: None,
            file_size_bytes,
            text_encoding,
        };

        let row = Self::create_expander_row(&doc);
        list_box.append(&row);
        documents.borrow_mut().push(doc);

        Ok(())
    }

    fn scan_directory_local(dir: &std::path::Path) -> anyhow::Result<Vec<Document>> {
        let span = tracing::span!(tracing::Level::DEBUG, "scan_directory_local", dir = ?dir);
        let _enter = span.enter();
        let mut documents = Vec::new();
        if !dir.is_dir() {
            return Ok(documents);
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if matches!(ext_lower.as_str(), "txt" | "epub" | "md" | "markdown") {
                        let title = path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        let format = DocumentScanner::normalize_format(&ext_lower);
                        let file_size_bytes = std::fs::metadata(&path).ok().map(|m| m.len() as i64);
                        let text_encoding = match ext_lower.as_str() {
                            "txt" | "md" | "markdown" => Some("UTF-8".to_string()),
                            _ => None,
                        };

                        documents.push(Document {
                            id: -1,
                            title,
                            author: Some("Unknown".to_string()),
                            format,
                            path: path.to_str().unwrap_or_default().to_string(),
                            date_added: chrono::Utc::now().to_rfc3339(),
                            last_opened: None,
                            cover_path: None,
                            file_size_bytes,
                            text_encoding,
                        });
                    }
                }
            }
        }
        Ok(documents)
    }

    fn create_expander_row(doc: &Document) -> adw::ExpanderRow {
        let expander = adw::ExpanderRow::builder()
            .title(&doc.title)
            .subtitle(&Self::format_subtitle(
                &doc.format,
                doc.file_size_bytes,
                &doc.text_encoding,
            ))
            .selectable(false)
            .show_enable_switch(false)
            .build();

        let path_row = adw::ActionRow::builder()
            .title(&i18n::translate("PATH"))
            .subtitle(&doc.path)
            .build();
        expander.add_row(&path_row);

        let format_row = adw::ActionRow::builder()
            .title(&i18n::translate("FORMAT"))
            .subtitle(&doc.format)
            .build();
        expander.add_row(&format_row);

        let size_str = Self::format_size(doc.file_size_bytes);
        let size_row = adw::ActionRow::builder()
            .title(&i18n::translate("SIZE"))
            .subtitle(&size_str)
            .build();
        expander.add_row(&size_row);

        if let Some(ref encoding) = doc.text_encoding {
            let encoding_row = adw::ActionRow::builder()
                .title(&i18n::translate("ENCODING"))
                .subtitle(encoding)
                .build();
            expander.add_row(&encoding_row);
        }

        expander
    }

    fn format_subtitle(
        format: &str,
        file_size_bytes: Option<i64>,
        text_encoding: &Option<String>,
    ) -> String {
        let size_str = Self::format_size(file_size_bytes);
        let mut parts = Vec::new();
        if !format.is_empty() {
            parts.push(format.to_string());
        }
        if !size_str.is_empty() {
            parts.push(size_str);
        }
        if let Some(ref enc) = text_encoding {
            parts.push(enc.clone());
        }
        parts.join(" · ")
    }

    fn format_size(bytes: Option<i64>) -> String {
        let bytes = match bytes {
            Some(b) if b > 0 => b,
            _ => return String::new(),
        };
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;
        let b = bytes as f64;
        if b >= GB {
            format!("{:.1} GB", b / GB)
        } else if b >= MB {
            format!("{:.1} MB", b / MB)
        } else if b >= KB {
            format!("{:.1} KB", b / KB)
        } else {
            format!("{} B", bytes)
        }
    }

    pub fn update_ui_strings(&self) {
        self.convert_button.set_label(&i18n::translate("CONVERT"));
        self.status_page.set_title(&i18n::translate("NO_FILES"));
        self.status_page
            .set_description(Some(&i18n::translate("IMPORT_FILES_TO_GET_STARTED")));
    }

    fn show_error_dialog(&self, title: &str, message: &str) {
        let dialog = gtk::MessageDialog::new(
            self.obj()
                .root()
                .as_ref()
                .and_then(|w| w.downcast_ref::<gtk::Window>()),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Error,
            gtk::ButtonsType::Ok,
            "{}",
        );
        dialog.set_title(Some(title));
        dialog.set_secondary_text(Some(message));
        dialog.connect_response(|dialog, _| dialog.close());
        dialog.show();
    }

    fn show_warning(&self, parent: &gtk::Window, message: &str) {
        let dialog = gtk::MessageDialog::new(
            Some(parent),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Warning,
            gtk::ButtonsType::Ok,
            "{}",
        );
        dialog.set_title(Some(&i18n::translate("WARNING")));
        dialog.set_secondary_text(Some(message));
        dialog.connect_response(|dialog, _| dialog.close());
        dialog.show();
    }

    fn update_convert_button(&self) {
        let has_docs = !self.documents.borrow().is_empty();
        tracing::debug!("update_convert_button: has_docs={}", has_docs);
        self.convert_button.set_sensitive(has_docs);

        // Always show import overlay with buttons
        self.import_overlay.set_visible(true);

        // Switch between list and status pages in the stack
        if has_docs {
            tracing::debug!("Setting visible child to list");
            self.content_stack.set_visible_child_name("list");
        } else {
            tracing::debug!("Setting visible child to status");
            self.content_stack.set_visible_child_name("status");
        }
    }
}

impl ObjectImpl for ConvertWidget {
    fn constructed(&self) {
        self.parent_constructed();

        // Load saved export format from settings
        let saved_format = if let Some(db) = self.db.borrow().as_ref() {
            let mgr = crate::core::SettingsManager::new(db.clone());
            mgr.get_export_format()
        } else {
            "txt".to_string()
        };

        *self.selected_format.borrow_mut() = saved_format.clone();

        // Set the toggle to the saved format
        self.output_toggle.set_active_name(Some(&saved_format));

        // Connect to format changes
        self.output_toggle.connect_active_name_notify(|toggle| {
            let widget = toggle
                .ancestor(super::ConvertWidget::static_type())
                .and_then(|w| w.downcast::<super::ConvertWidget>().ok());
            if let Some(widget) = widget {
                let name = toggle.active_name().unwrap_or_default().to_string();
                *widget.imp().selected_format.borrow_mut() = name.clone();

                // Save the format to settings
                if let Some(db) = widget.imp().db.borrow().as_ref() {
                    let mut mgr = crate::core::SettingsManager::new(db.clone());
                    if let Err(e) = mgr.set_export_format(&name) {
                        tracing::error!("Failed to save export format: {}", e);
                    }
                }

                tracing::info!(format = %name, "Output format changed and saved");
            }
        });

        // Initialize UI state
        tracing::debug!(
            "ConvertWidget constructed, documents count: {}",
            self.documents.borrow().len()
        );
        self.update_convert_button();
        tracing::debug!(
            "Initial visible child: {:?}",
            self.content_stack.visible_child_name()
        );
    }
}
impl WidgetImpl for ConvertWidget {}
impl BoxImpl for ConvertWidget {}
