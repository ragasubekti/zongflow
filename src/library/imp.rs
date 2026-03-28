use crate::database::Database;
use crate::document_object::DocumentObject;
use crate::i18n;
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use std::cell::RefCell;
use std::path::Path;
use tracing;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/github/zongflow/library.ui")]
pub struct LibraryWidget {
    #[template_child]
    pub view_stack: TemplateChild<adw::ViewStack>,
    #[template_child]
    pub list_scrolled: TemplateChild<gtk::ScrolledWindow>,
    #[template_child]
    pub grid_scrolled: TemplateChild<gtk::ScrolledWindow>,
    #[template_child]
    pub status_page: TemplateChild<adw::StatusPage>,
    pub db: RefCell<Option<Database>>,
    pub store: RefCell<Option<gio::ListStore>>,
    pub column_view: RefCell<Option<gtk::ColumnView>>,
    pub grid_view: RefCell<Option<gtk::GridView>>,
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

impl ObjectImpl for LibraryWidget {}
impl WidgetImpl for LibraryWidget {}
impl BoxImpl for LibraryWidget {}

impl LibraryWidget {
    fn setup_views(&self) {
        let store = gio::ListStore::new::<DocumentObject>();

        // --- ColumnView (list) ---
        let selection = gtk::NoSelection::new(Some(store.clone()));
        let column_view = gtk::ColumnView::new(Some(selection));
        column_view.set_show_column_separators(true);
        column_view.set_show_row_separators(true);

        // Cover column
        let cover_factory = gtk::SignalListItemFactory::new();
        let cover_col = gtk::ColumnViewColumn::new(
            Some(&i18n::translate("Cover")),
            Some(cover_factory.clone()),
        );
        cover_col.set_fixed_width(48);
        cover_factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let picture = gtk::Picture::new();
            picture.set_width_request(36);
            picture.set_height_request(48);
            picture.set_content_fit(gtk::ContentFit::Cover);
            picture.set_can_shrink(true);
            item.set_child(Some(&picture));
        });
        cover_factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let obj = item.item().and_downcast::<DocumentObject>().unwrap();
            let picture = item.child().and_downcast::<gtk::Picture>().unwrap();
            match obj.cover_path() {
                Some(ref cover) if !cover.is_empty() && Path::new(cover).exists() => {
                    picture.set_filename(Some(cover.as_str()));
                }
                _ => {
                    picture.set_filename(None::<&str>);
                }
            }
        });
        column_view.append_column(&cover_col);

        // Title column (title + muted path)
        let title_factory = gtk::SignalListItemFactory::new();
        let title_col = gtk::ColumnViewColumn::new(
            Some(&i18n::translate("Title")),
            Some(title_factory.clone()),
        );
        title_col.set_expand(true);
        title_factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 2);
            let title_label = gtk::Label::new(None);
            title_label.set_halign(gtk::Align::Start);
            title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            title_label.add_css_class("title");
            let path_label = gtk::Label::new(None);
            path_label.set_halign(gtk::Align::Start);
            path_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            path_label.add_css_class("dim-label");
            path_label.add_css_class("caption");
            vbox.append(&title_label);
            vbox.append(&path_label);
            item.set_child(Some(&vbox));
        });
        title_factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let obj = item.item().and_downcast::<DocumentObject>().unwrap();
            let vbox = item.child().and_downcast::<gtk::Box>().unwrap();
            let title_lbl = vbox.first_child().and_downcast::<gtk::Label>().unwrap();
            let path_lbl = title_lbl
                .next_sibling()
                .and_downcast::<gtk::Label>()
                .unwrap();
            title_lbl.set_text(&obj.title());
            path_lbl.set_text(&obj.display_path());
        });
        column_view.append_column(&title_col);

        // Author column
        let author_factory = gtk::SignalListItemFactory::new();
        let author_col = gtk::ColumnViewColumn::new(
            Some(&i18n::translate("Author")),
            Some(author_factory.clone()),
        );
        author_factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let label = gtk::Label::new(None);
            label.set_halign(gtk::Align::Start);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            item.set_child(Some(&label));
        });
        author_factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let obj = item.item().and_downcast::<DocumentObject>().unwrap();
            let label = item.child().and_downcast::<gtk::Label>().unwrap();
            label.set_text(&obj.display_author());
        });
        column_view.append_column(&author_col);

        // Date column
        let date_factory = gtk::SignalListItemFactory::new();
        let date_col =
            gtk::ColumnViewColumn::new(Some(&i18n::translate("Date")), Some(date_factory.clone()));
        date_factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let label = gtk::Label::new(None);
            label.set_halign(gtk::Align::Start);
            item.set_child(Some(&label));
        });
        date_factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let obj = item.item().and_downcast::<DocumentObject>().unwrap();
            let label = item.child().and_downcast::<gtk::Label>().unwrap();
            label.set_text(&obj.format_date());
        });
        column_view.append_column(&date_col);

        // Size column
        let size_factory = gtk::SignalListItemFactory::new();
        let size_col =
            gtk::ColumnViewColumn::new(Some(&i18n::translate("Size")), Some(size_factory.clone()));
        size_factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let label = gtk::Label::new(None);
            label.set_halign(gtk::Align::End);
            item.set_child(Some(&label));
        });
        size_factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let obj = item.item().and_downcast::<DocumentObject>().unwrap();
            let label = item.child().and_downcast::<gtk::Label>().unwrap();
            label.set_text(&obj.format_size());
        });
        column_view.append_column(&size_col);

        // Format column
        let format_factory = gtk::SignalListItemFactory::new();
        let format_col = gtk::ColumnViewColumn::new(
            Some(&i18n::translate("Format")),
            Some(format_factory.clone()),
        );
        format_factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let label = gtk::Label::new(None);
            label.set_halign(gtk::Align::Start);
            label.add_css_class("tag");
            item.set_child(Some(&label));
        });
        format_factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let obj = item.item().and_downcast::<DocumentObject>().unwrap();
            let label = item.child().and_downcast::<gtk::Label>().unwrap();
            label.set_text(&obj.format());
        });
        column_view.append_column(&format_col);

        self.list_scrolled.set_child(Some(&column_view));

        // --- GridView (grid) ---
        let grid_selection = gtk::NoSelection::new(Some(store.clone()));
        let grid_factory = gtk::SignalListItemFactory::new();
        let grid_view = gtk::GridView::new(Some(grid_selection), Some(grid_factory.clone()));
        grid_view.set_min_columns(3);
        grid_view.set_max_columns(8);

        grid_factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let card = gtk::Box::new(gtk::Orientation::Vertical, 6);
            card.set_halign(gtk::Align::Center);
            card.add_css_class("card");
            card.set_margin_top(4);
            card.set_margin_bottom(4);
            card.set_margin_start(4);
            card.set_margin_end(4);

            let picture = gtk::Picture::new();
            picture.set_width_request(120);
            picture.set_height_request(160);
            picture.set_content_fit(gtk::ContentFit::Cover);
            picture.set_can_shrink(true);
            picture.add_css_class("card");

            let title_label = gtk::Label::new(None);
            title_label.set_halign(gtk::Align::Center);
            title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            title_label.set_max_width_chars(15);
            title_label.add_css_class("caption");
            title_label.add_css_class("bold");

            let author_label = gtk::Label::new(None);
            author_label.set_halign(gtk::Align::Center);
            author_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            author_label.add_css_class("dim-label");
            author_label.add_css_class("caption");

            let meta_label = gtk::Label::new(None);
            meta_label.set_halign(gtk::Align::Center);
            meta_label.add_css_class("dim-label");
            meta_label.add_css_class("caption");

            card.append(&picture);
            card.append(&title_label);
            card.append(&author_label);
            card.append(&meta_label);
            item.set_child(Some(&card));
        });

        grid_factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let obj = item.item().and_downcast::<DocumentObject>().unwrap();
            let card = item.child().and_downcast::<gtk::Box>().unwrap();
            let picture = card.first_child().and_downcast::<gtk::Picture>().unwrap();
            let title_lbl = picture.next_sibling().and_downcast::<gtk::Label>().unwrap();
            let author_lbl = title_lbl
                .next_sibling()
                .and_downcast::<gtk::Label>()
                .unwrap();
            let meta_lbl = author_lbl
                .next_sibling()
                .and_downcast::<gtk::Label>()
                .unwrap();

            match obj.cover_path() {
                Some(ref cover) if !cover.is_empty() && Path::new(cover).exists() => {
                    picture.set_filename(Some(cover.as_str()));
                }
                _ => {
                    picture.set_filename(None::<&str>);
                }
            }
            title_lbl.set_text(&obj.title());
            author_lbl.set_text(&obj.display_author());
            let size = obj.format_size();
            let fmt = obj.format();
            let meta = if !size.is_empty() {
                format!("{} · {}", size, fmt)
            } else {
                fmt.clone()
            };
            meta_lbl.set_text(&meta);
        });

        self.grid_scrolled.set_child(Some(&grid_view));

        // Store references for later access
        *self.store.borrow_mut() = Some(store);
        *self.column_view.borrow_mut() = Some(column_view);
        *self.grid_view.borrow_mut() = Some(grid_view);
    }

    pub fn load_documents(&self) {
        if self.store.borrow().is_none() {
            self.setup_views();
        }

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

        let store = self.store.borrow();
        let store = store.as_ref().unwrap();
        store.remove_all();

        for doc in &docs {
            let obj = DocumentObject::from(doc);
            store.append(&obj);
        }

        // Toggle empty state
        let is_empty = docs.is_empty();
        self.status_page.set_visible(is_empty);
        self.view_stack.set_visible(!is_empty);
    }

    pub fn update_ui_strings(&self) {
        // No more title_label — window header title is used instead
    }
}
