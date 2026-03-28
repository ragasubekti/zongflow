use std::cell::RefCell;

use chrono::DateTime;
use glib::prelude::*;
use glib::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;

    #[derive(Debug, glib::Properties)]
    #[properties(wrapper_type = super::DocumentObject)]
    pub struct DocumentObject {
        #[property(get, set)]
        pub id: RefCell<i64>,
        #[property(get, set)]
        pub title: RefCell<String>,
        #[property(get, set, nullable)]
        pub author: RefCell<Option<String>>,
        #[property(get, set)]
        pub format: RefCell<String>,
        #[property(get, set)]
        pub path: RefCell<String>,
        #[property(get, set)]
        pub date_added: RefCell<String>,
        #[property(get, set, nullable)]
        pub last_opened: RefCell<Option<String>>,
        #[property(get, set, nullable)]
        pub cover_path: RefCell<Option<String>>,
        #[property(get, set)]
        pub file_size_bytes: RefCell<i64>,
        #[property(get, set, nullable)]
        pub text_encoding: RefCell<Option<String>>,
    }

    impl Default for DocumentObject {
        fn default() -> Self {
            Self {
                id: RefCell::new(0),
                title: RefCell::new(String::new()),
                author: RefCell::new(None),
                format: RefCell::new(String::new()),
                path: RefCell::new(String::new()),
                date_added: RefCell::new(String::new()),
                last_opened: RefCell::new(None),
                cover_path: RefCell::new(None),
                file_size_bytes: RefCell::new(0),
                text_encoding: RefCell::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DocumentObject {
        const NAME: &'static str = "DocumentObject";
        type Type = super::DocumentObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for DocumentObject {}

    impl DocumentObject {}
}

glib::wrapper! {
    pub struct DocumentObject(ObjectSubclass<imp::DocumentObject>);
}

impl DocumentObject {
    pub fn new(
        id: i64,
        title: &str,
        author: Option<&str>,
        format: &str,
        path: &str,
        date_added: &str,
        last_opened: Option<&str>,
        cover_path: Option<&str>,
        file_size_bytes: Option<i64>,
        text_encoding: Option<&str>,
    ) -> Self {
        glib::Object::builder()
            .property("id", id)
            .property("title", title)
            .property("author", author.unwrap_or(""))
            .property("format", format)
            .property("path", path)
            .property("date-added", date_added)
            .property("last-opened", last_opened.unwrap_or(""))
            .property("cover-path", cover_path.unwrap_or(""))
            .property("file-size-bytes", file_size_bytes.unwrap_or(0))
            .property("text-encoding", text_encoding.unwrap_or(""))
            .build()
    }

    /// Format file size to human-readable string
    pub fn format_size(&self) -> String {
        let bytes = self.file_size_bytes();
        if bytes <= 0 {
            return String::new();
        }
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

    /// Format date to human-readable: "23 May 2025 22:44"
    pub fn format_date(&self) -> String {
        let date_str = self.date_added();
        if date_str.is_empty() {
            return String::new();
        }
        match DateTime::parse_from_rfc3339(&date_str) {
            Ok(dt) => dt.format("%d %b %Y %H:%M").to_string(),
            Err(_) => date_str,
        }
    }

    /// Get display author, returning "Unknown" if missing
    /// Get display author, returning "Unknown" if missing
    pub fn display_author(&self) -> String {
        match self.author() {
            Some(a) if !a.is_empty() => a,
            _ => crate::i18n::translate("Unknown"),
        }
    }

    /// Get path display label (muted)
    pub fn display_path(&self) -> String {
        let p = self.path();
        // Show just the filename
        std::path::Path::new(&p)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or(p)
    }
}

impl From<&crate::database::Document> for DocumentObject {
    fn from(doc: &crate::database::Document) -> Self {
        DocumentObject::new(
            doc.id,
            &doc.title,
            doc.author.as_deref(),
            &doc.format,
            &doc.path,
            &doc.date_added,
            doc.last_opened.as_deref(),
            doc.cover_path.as_deref(),
            doc.file_size_bytes,
            doc.text_encoding.as_deref(),
        )
    }
}
