mod item_row;

use libadwaita as adw;
use adw::prelude::*;
use gtk4::prelude::*;
use chrono::{DateTime, Local};
use everything_core::{Database, FileCategory, FileEntry, FileWatcher, SearchQuery};
use human_bytes::human_bytes;
use item_row::FileObject;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

const APP_ID: &str = "com.everything.linux";
const APP_VERSION: &str = "1.0.0";
const AUTHOR_NAME: &str = "Arghadeep Laskar";
const AUTHOR_GITHUB: &str = "https://github.com/arghadeeplaskar";

fn main() -> glib::ExitCode {
    let app = adw::Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| {
        load_custom_styles();
    });

    app.connect_activate(build_ui);
    app.run()
}

fn load_custom_styles() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        "
        /* Unified Search & Filter Card */
        .control-panel {
            background-color: alpha(@window_fg_color, 0.035);
            border: 1px solid alpha(@window_fg_color, 0.08);
            border-radius: 8px;
            margin: 8px 12px 6px 12px;
            padding: 8px 10px;
        }

        /* Search input */
        .main-search-entry {
            border-radius: 6px;
            font-size: 13px;
        }

        /* Flag buttons (Match Case & Regex) */
        .flag-btn {
            font-weight: 700;
            font-family: monospace;
            font-size: 12px;
            padding: 2px 10px;
            min-height: 32px;
        }

        /* Category Segmented Bar (.linked gives tight connected segments) */
        .category-bar button {
            font-size: 12px;
            font-weight: 500;
            padding: 4px 12px;
            min-height: 28px;
            border-radius: 0;
            border-color: alpha(@window_fg_color, 0.12);
        }
        .category-bar button:first-child {
            border-top-left-radius: 6px;
            border-bottom-left-radius: 6px;
        }
        .category-bar button:last-child {
            border-top-right-radius: 6px;
            border-bottom-right-radius: 6px;
        }
        .category-bar button:checked {
            background-color: @accent_bg_color;
            color: @accent_fg_color;
        }

        /* Status Bar */
        .footer-bar {
            border-top: 1px solid alpha(@window_fg_color, 0.08);
            padding: 6px 14px;
            background-color: alpha(@window_fg_color, 0.02);
        }
        .status-pill {
            color: #2ec27e;
            font-weight: 600;
            font-size: 11px;
            background-color: alpha(#2ec27e, 0.12);
            padding: 2px 8px;
            border-radius: 9999px;
        }
        .dim-path {
            color: alpha(@window_fg_color, 0.55);
            font-size: 12px;
        }
        .tabular-text {
            font-family: monospace;
            font-size: 11.5px;
            color: alpha(@window_fg_color, 0.75);
        }

        /* List Rows */
        columnview row {
            min-height: 34px;
            padding: 1px 6px;
            border-bottom: 1px solid alpha(@window_fg_color, 0.04);
            transition: background-color 120ms ease-in-out;
        }
        columnview row:hover {
            background-color: alpha(@accent_bg_color, 0.09);
        }
        columnview row:selected {
            background-color: alpha(@accent_bg_color, 0.28);
        }
        .hidden-file {
            opacity: 0.65;
        }

        /* Custom Compact About Modal */
        .about-window {
            background-color: @window_bg_color;
            border-radius: 12px;
        }
        .about-card {
            padding: 22px 20px 18px 20px;
        }
        .about-app-title {
            font-size: 18px;
            font-weight: 700;
        }
        .about-version-tag {
            font-size: 11px;
            font-weight: 600;
            color: @accent_color;
            background-color: alpha(@accent_bg_color, 0.15);
            padding: 2px 8px;
            border-radius: 9999px;
        }
        .about-author-card {
            background-color: alpha(@window_fg_color, 0.04);
            border: 1px solid alpha(@window_fg_color, 0.08);
            border-radius: 8px;
            padding: 10px 14px;
        }
        .about-author-name {
            font-size: 14px;
            font-weight: 600;
        }
        .about-github-btn {
            font-size: 12px;
            font-weight: 500;
            padding: 4px 8px;
        }
        .about-stats-pill {
            font-size: 11px;
            color: alpha(@window_fg_color, 0.65);
            background-color: alpha(@window_fg_color, 0.035);
            border: 1px solid alpha(@window_fg_color, 0.06);
            border-radius: 6px;
            padding: 4px 8px;
        }
        .about-footer {
            font-size: 11px;
            color: alpha(@window_fg_color, 0.45);
        }
        ",
    );

    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn get_icon_name(entry: &FileEntry) -> &'static str {
    if entry.is_dir {
        return "folder";
    }
    let ext = entry.extension().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        // Specific Languages & Scripts
        "rs" => "text-x-script",
        "py" => "text-x-python",
        "md" => "text-markdown",
        "c" | "cpp" | "h" | "hpp" => "text-x-script",
        "js" | "ts" | "jsx" | "tsx" => "text-x-script",
        "html" | "htm" => "text-html",
        "css" | "scss" | "sass" => "text-x-script",
        "json" | "toml" | "yaml" | "yml" | "xml" | "sql" => "text-x-script",
        "sh" | "bash" | "zsh" => "text-x-script",
        "go" | "java" | "kt" | "swift" | "php" => "text-x-script",

        // Images
        "png" | "jpg" | "jpeg" | "svg" | "gif" | "webp" | "bmp" | "ico" | "tiff" => {
            "image-x-generic"
        }

        // Audio
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "opus" | "wma" => "audio-x-generic",

        // Video
        "mp4" | "mkv" | "webm" | "avi" | "mov" | "flv" | "wmv" => "video-x-generic",

        // Documents
        "pdf" => "application-pdf",
        "doc" | "docx" | "odt" | "rtf" => "x-office-document",
        "xls" | "xlsx" | "ods" | "csv" => "x-office-spreadsheet",
        "ppt" | "pptx" | "odp" => "x-office-presentation",
        "txt" | "log" => "text-x-generic",

        // Archives
        "zip" | "tar" | "gz" | "7z" | "rar" | "xz" | "bz2" | "zst" | "deb" | "rpm" | "iso" => {
            "package-x-generic"
        }

        // Executables / Binaries
        "bin" | "so" | "o" | "a" | "appimage" => "application-x-executable",

        // Fonts
        "ttf" | "otf" | "woff" | "woff2" => "font-x-generic",

        _ => "text-x-generic",
    }
}

fn build_ui(app: &adw::Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Everything Linux")
        .default_width(1150)
        .default_height(720)
        .build();

    let root_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    // ==========================================
    // 1. Header Bar
    // ==========================================
    let header_bar = adw::HeaderBar::new();
    let title_widget = adw::WindowTitle::new("Everything Linux", "Initializing database...");
    header_bar.set_title_widget(Some(&title_widget));

    // Spinner for indexing activity
    let index_spinner = gtk4::Spinner::new();
    index_spinner.start();
    index_spinner.set_margin_end(6);

    // Refresh / Rescan button
    let rescan_button = gtk4::Button::builder()
        .icon_name("view-refresh-symbolic")
        .tooltip_text("Re-index Filesystem (F5)")
        .build();

    let left_header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    left_header_box.append(&rescan_button);
    left_header_box.append(&index_spinner);
    header_bar.pack_start(&left_header_box);

    // Menu button on right
    let menu_button = gtk4::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .tooltip_text("Main Menu")
        .build();

    let menu_popover = gtk4::Popover::new();
    let menu_content = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    menu_content.set_margin_start(4);
    menu_content.set_margin_end(4);
    menu_content.set_margin_top(4);
    menu_content.set_margin_bottom(4);

    let btn_shortcuts = gtk4::Button::builder()
        .label("Keyboard Shortcuts")
        .has_frame(false)
        .build();
    let btn_about = gtk4::Button::builder()
        .label("About Everything Linux")
        .has_frame(false)
        .build();

    menu_content.append(&btn_shortcuts);
    menu_content.append(&btn_about);
    menu_popover.set_child(Some(&menu_content));
    menu_button.set_popover(Some(&menu_popover));
    header_bar.pack_end(&menu_button);

    root_box.append(&header_bar);

    // ==========================================
    // 2. Unified Control Panel (Search + Categorized Filters)
    // ==========================================
    let control_panel = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    control_panel.add_css_class("control-panel");

    // Row A: Search Input + Flag Toggles
    let search_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

    let search_entry = gtk4::SearchEntry::builder()
        .placeholder_text("Search files, folders, extensions, or wildcards (* and ?)...")
        .hexpand(true)
        .css_classes(["main-search-entry"])
        .build();
    search_row.append(&search_entry);

    // Linked group for Case and Regex flags
    let flags_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    flags_box.add_css_class("linked");

    let match_case_btn = gtk4::ToggleButton::builder()
        .label("Aa")
        .tooltip_text("Match Case (Case Sensitive)")
        .css_classes(["flag-btn"])
        .build();
    let regex_btn = gtk4::ToggleButton::builder()
        .label(".*")
        .tooltip_text("Regular Expression")
        .css_classes(["flag-btn"])
        .build();

    flags_box.append(&match_case_btn);
    flags_box.append(&regex_btn);
    search_row.append(&flags_box);

    control_panel.append(&search_row);

    // Hairline Separator
    let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    sep.set_margin_top(2);
    sep.set_margin_bottom(2);
    control_panel.append(&sep);

    // Row B: Segmented Category Bar
    let category_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);

    let category_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    category_bar.add_css_class("linked");
    category_bar.add_css_class("category-bar");

    let active_category = Rc::new(RefCell::new(FileCategory::All));

    // Categories with clean labels and official symbolic icons
    let categories: [(&str, &str, FileCategory); 8] = [
        ("view-grid-symbolic", "All", FileCategory::All),
        ("folder-symbolic", "Folders", FileCategory::Folders),
        ("x-office-document-symbolic", "Documents", FileCategory::Documents),
        ("image-x-generic-symbolic", "Images", FileCategory::Images),
        ("audio-x-generic-symbolic", "Audio", FileCategory::Audio),
        ("video-x-generic-symbolic", "Videos", FileCategory::Video),
        ("package-x-generic-symbolic", "Archives", FileCategory::Archives),
        ("text-x-script-symbolic", "Code", FileCategory::Code),
    ];

    let mut category_buttons = Vec::new();

    // First button (All)
    let first_btn = gtk4::ToggleButton::builder().active(true).build();
    let first_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 5);
    let first_icon = gtk4::Image::from_icon_name(categories[0].0);
    let first_lbl = gtk4::Label::new(Some(categories[0].1));
    first_box.append(&first_icon);
    first_box.append(&first_lbl);
    first_btn.set_child(Some(&first_box));
    category_bar.append(&first_btn);
    category_buttons.push((first_btn.clone(), categories[0].2));

    // Remaining buttons linked in group
    for item in &categories[1..] {
        let (icon_name, label_text, cat) = *item;
        let btn = gtk4::ToggleButton::builder().group(&first_btn).build();
        let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 5);
        let icon = gtk4::Image::from_icon_name(icon_name);
        let lbl = gtk4::Label::new(Some(label_text));
        hbox.append(&icon);
        hbox.append(&lbl);
        btn.set_child(Some(&hbox));
        category_bar.append(&btn);
        category_buttons.push((btn, cat));
    }

    category_row.append(&category_bar);

    // Spacer
    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    category_row.append(&spacer);

    control_panel.append(&category_row);
    root_box.append(&control_panel);

    // ==========================================
    // 3. Virtualized ColumnView Table
    // ==========================================
    let list_store = gio::ListStore::new::<FileObject>();
    let column_view = gtk4::ColumnView::new(Option::<gtk4::SelectionModel>::None);
    column_view.set_show_row_separators(true);
    column_view.set_show_column_separators(true);

    let sort_model = gtk4::SortListModel::new(Some(list_store.clone()), column_view.sorter());
    let selection_model = gtk4::SingleSelection::new(Some(sort_model));
    column_view.set_model(Some(&selection_model));

    // Column 1: Name (Icon + Filename)
    let col1_factory = gtk4::SignalListItemFactory::new();
    col1_factory.connect_setup(|_, list_item| {
        let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let icon = gtk4::Image::new();
        icon.set_pixel_size(18);
        let label = gtk4::Label::builder()
            .xalign(0.0)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();
        hbox.append(&icon);
        hbox.append(&label);
        let item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
        item.set_child(Some(&hbox));
    });
    col1_factory.connect_bind(|_, list_item| {
        let item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
        if let Some(file_obj) = item.item().and_downcast::<FileObject>() {
            if let Some(entry) = file_obj.entry() {
                let hbox = item.child().unwrap().downcast::<gtk4::Box>().unwrap();
                let icon = hbox.first_child().unwrap().downcast::<gtk4::Image>().unwrap();
                let label = hbox.last_child().unwrap().downcast::<gtk4::Label>().unwrap();

                icon.set_icon_name(Some(get_icon_name(&entry)));
                label.set_text(&entry.name);

                if entry.name.starts_with('.') {
                    label.add_css_class("hidden-file");
                } else {
                    label.remove_css_class("hidden-file");
                }
            }
        }
    });
    let sorter1 = gtk4::CustomSorter::new(|a, b| {
        let fa = a.downcast_ref::<FileObject>().and_then(|o| o.entry());
        let fb = b.downcast_ref::<FileObject>().and_then(|o| o.entry());
        match (fa, fb) {
            (Some(ea), Some(eb)) => ea.name_lower.cmp(&eb.name_lower).into(),
            _ => gtk4::Ordering::Equal,
        }
    });
    let col1 = gtk4::ColumnViewColumn::builder()
        .title("Name")
        .factory(&col1_factory)
        .sorter(&sorter1)
        .expand(true)
        .build();
    column_view.append_column(&col1);

    // Column 2: Path
    let col2_factory = gtk4::SignalListItemFactory::new();
    col2_factory.connect_setup(|_, list_item| {
        let label = gtk4::Label::builder()
            .xalign(0.0)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .css_classes(["dim-path"])
            .build();
        let item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
        item.set_child(Some(&label));
    });
    col2_factory.connect_bind(|_, list_item| {
        let item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
        if let Some(file_obj) = item.item().and_downcast::<FileObject>() {
            if let Some(entry) = file_obj.entry() {
                let label = item.child().unwrap().downcast::<gtk4::Label>().unwrap();
                label.set_text(&entry.parent.to_string_lossy());
            }
        }
    });
    let sorter2 = gtk4::CustomSorter::new(|a, b| {
        let fa = a.downcast_ref::<FileObject>().and_then(|o| o.entry());
        let fb = b.downcast_ref::<FileObject>().and_then(|o| o.entry());
        match (fa, fb) {
            (Some(ea), Some(eb)) => ea.parent.cmp(&eb.parent).into(),
            _ => gtk4::Ordering::Equal,
        }
    });
    let col2 = gtk4::ColumnViewColumn::builder()
        .title("Path")
        .factory(&col2_factory)
        .sorter(&sorter2)
        .expand(true)
        .build();
    column_view.append_column(&col2);

    // Column 3: Size
    let col3_factory = gtk4::SignalListItemFactory::new();
    col3_factory.connect_setup(|_, list_item| {
        let label = gtk4::Label::builder()
            .xalign(1.0)
            .css_classes(["tabular-text"])
            .build();
        let item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
        item.set_child(Some(&label));
    });
    col3_factory.connect_bind(|_, list_item| {
        let item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
        if let Some(file_obj) = item.item().and_downcast::<FileObject>() {
            if let Some(entry) = file_obj.entry() {
                let label = item.child().unwrap().downcast::<gtk4::Label>().unwrap();
                if entry.is_dir {
                    label.set_text("");
                } else {
                    label.set_text(&human_bytes(entry.size as f64));
                }
            }
        }
    });
    let sorter3 = gtk4::CustomSorter::new(|a, b| {
        let fa = a.downcast_ref::<FileObject>().and_then(|o| o.entry());
        let fb = b.downcast_ref::<FileObject>().and_then(|o| o.entry());
        match (fa, fb) {
            (Some(ea), Some(eb)) => ea.size.cmp(&eb.size).into(),
            _ => gtk4::Ordering::Equal,
        }
    });
    let col3 = gtk4::ColumnViewColumn::builder()
        .title("Size")
        .factory(&col3_factory)
        .sorter(&sorter3)
        .fixed_width(110)
        .build();
    column_view.append_column(&col3);

    // Column 4: Date Modified
    let col4_factory = gtk4::SignalListItemFactory::new();
    col4_factory.connect_setup(|_, list_item| {
        let label = gtk4::Label::builder()
            .xalign(0.0)
            .css_classes(["tabular-text"])
            .build();
        let item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
        item.set_child(Some(&label));
    });
    col4_factory.connect_bind(|_, list_item| {
        let item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
        if let Some(file_obj) = item.item().and_downcast::<FileObject>() {
            if let Some(entry) = file_obj.entry() {
                let label = item.child().unwrap().downcast::<gtk4::Label>().unwrap();
                if entry.modified > 0 {
                    let dt = DateTime::from_timestamp(entry.modified as i64, 0)
                        .unwrap_or_default()
                        .with_timezone(&Local);
                    label.set_text(&dt.format("%Y-%m-%d %H:%M").to_string());
                } else {
                    label.set_text("");
                }
            }
        }
    });
    let sorter4 = gtk4::CustomSorter::new(|a, b| {
        let fa = a.downcast_ref::<FileObject>().and_then(|o| o.entry());
        let fb = b.downcast_ref::<FileObject>().and_then(|o| o.entry());
        match (fa, fb) {
            (Some(ea), Some(eb)) => ea.modified.cmp(&eb.modified).into(),
            _ => gtk4::Ordering::Equal,
        }
    });
    let col4 = gtk4::ColumnViewColumn::builder()
        .title("Date Modified")
        .factory(&col4_factory)
        .sorter(&sorter4)
        .fixed_width(150)
        .build();
    column_view.append_column(&col4);

    // Double-click row to open
    let selection_clone = selection_model.clone();
    column_view.connect_activate(move |_, position| {
        if let Some(item) = selection_clone.item(position) {
            if let Some(file_obj) = item.downcast_ref::<FileObject>() {
                if let Some(entry) = file_obj.entry() {
                    let _ = open::that(entry.full_path());
                }
            }
        }
    });

    // Right-click context menu
    let gesture = gtk4::GestureClick::new();
    gesture.set_button(3);
    let selection_menu_clone = selection_model.clone();
    let window_clone = window.clone();
    gesture.connect_pressed(move |_, _, x, y| {
        if let Some(selected_item) = selection_menu_clone.selected_item() {
            if let Some(file_obj) = selected_item.downcast_ref::<FileObject>() {
                if let Some(entry) = file_obj.entry() {
                    let popover = gtk4::Popover::new();
                    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
                    menu_box.set_margin_start(6);
                    menu_box.set_margin_end(6);
                    menu_box.set_margin_top(6);
                    menu_box.set_margin_bottom(6);

                    let full_path = entry.full_path();
                    let parent_path = entry.parent.clone();

                    let btn_open = gtk4::Button::builder()
                        .label("Open File")
                        .icon_name("document-open-symbolic")
                        .build();
                    let p1 = full_path.clone();
                    let pop_c1 = popover.clone();
                    btn_open.connect_clicked(move |_| {
                        let _ = open::that(&p1);
                        pop_c1.popdown();
                    });
                    menu_box.append(&btn_open);

                    let btn_folder = gtk4::Button::builder()
                        .label("Open Containing Folder")
                        .icon_name("folder-open-symbolic")
                        .build();
                    let p2 = parent_path.clone();
                    let pop_c2 = popover.clone();
                    btn_folder.connect_clicked(move |_| {
                        let _ = open::that(&p2);
                        pop_c2.popdown();
                    });
                    menu_box.append(&btn_folder);

                    let btn_copy = gtk4::Button::builder()
                        .label("Copy Full Path")
                        .icon_name("edit-copy-symbolic")
                        .build();
                    let p3 = full_path.clone();
                    let pop_c3 = popover.clone();
                    let display = gtk4::prelude::WidgetExt::display(&window_clone);
                    btn_copy.connect_clicked(move |_| {
                        display.clipboard().set_text(&p3.to_string_lossy());
                        pop_c3.popdown();
                    });
                    menu_box.append(&btn_copy);

                    popover.set_child(Some(&menu_box));
                    popover.set_pointing_to(Some(&gtk4::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
                    popover.set_parent(&window_clone);
                    popover.popup();
                }
            }
        }
    });
    column_view.add_controller(gesture);

    // Scrolled window for table
    let scrolled = gtk4::ScrolledWindow::builder()
        .child(&column_view)
        .vexpand(true)
        .hexpand(true)
        .build();
    root_box.append(&scrolled);

    // ==========================================
    // 4. Status Footer Bar
    // ==========================================
    let status_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    status_box.add_css_class("footer-bar");

    let status_label = gtk4::Label::builder()
        .label("Initializing indexing engine...")
        .xalign(0.0)
        .hexpand(true)
        .css_classes(["dim-path"])
        .build();
    status_box.append(&status_label);

    let live_sync_label = gtk4::Label::builder()
        .label("● Live Sync Active")
        .css_classes(["status-pill"])
        .build();
    status_box.append(&live_sync_label);

    root_box.append(&status_box);
    window.set_content(Some(&root_box));
    window.present();

    // ==========================================
    // 5. Database Backend & Concurrency
    // ==========================================
    let db = Arc::new(Database::default());
    let db_for_indexing = Arc::clone(&db);
    let title_clone = title_widget.clone();
    let status_clone = status_label.clone();
    let list_store_clone = list_store.clone();
    let spinner_clone = index_spinner.clone();

    let scan_path = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"));

    let (tx, rx) = async_channel::unbounded::<(usize, u128)>();

    let scan_path_bg = scan_path.clone();
    let tx_first = tx.clone();
    std::thread::spawn(move || {
        let stats = db_for_indexing.index_roots(&[scan_path_bg]);
        let _ = tx_first.send_blocking((stats.total_files + stats.total_dirs, stats.elapsed_ms));
    });

    let db_for_watcher = Arc::clone(&db);
    let scan_path_watcher = scan_path.clone();
    std::thread::spawn(move || {
        let _ = FileWatcher::start((*db_for_watcher).clone(), vec![scan_path_watcher]);
    });

    // Handle initial index completion
    let db_main = Arc::clone(&db);
    let current_total_items = Rc::new(RefCell::new(0usize));
    let current_total_clone = Rc::clone(&current_total_items);

    glib::spawn_future_local(async move {
        while let Ok((count, elapsed)) = rx.recv().await {
            *current_total_clone.borrow_mut() = count;
            spinner_clone.stop();
            spinner_clone.set_visible(false);

            title_clone.set_subtitle(&format!("Indexed {} items in {} ms", count, elapsed));
            status_clone.set_text(&format!(
                "{} items indexed in RAM (~{:.1} MB) • Ready to search",
                count,
                (db_main.memory_usage_bytes() as f64) / 1_048_576.0
            ));

            let initial = db_main.search(&SearchQuery {
                max_results: 200,
                ..Default::default()
            });
            list_store_clone.remove_all();
            let items: Vec<FileObject> = initial
                .entries
                .into_iter()
                .map(FileObject::new)
                .collect();
            list_store_clone.extend_from_slice(&items);
        }
    });

    // Search query performer
    let db_search = Arc::clone(&db);
    let list_store_search = list_store.clone();
    let status_search = status_label.clone();
    let match_case_state = match_case_btn.clone();
    let regex_state = regex_btn.clone();
    let category_state = Rc::clone(&active_category);

    let perform_search = Rc::new(move |query_text: String| {
        let cat = *category_state.borrow();
        let q = SearchQuery {
            raw: query_text,
            case_sensitive: match_case_state.is_active(),
            is_regex: regex_state.is_active(),
            is_wildcard: false,
            dir_only: false,
            file_only: false,
            category: cat,
            max_results: 1000,
        };

        let res = db_search.search(&q);
        list_store_search.remove_all();

        let objects: Vec<FileObject> = res
            .entries
            .into_iter()
            .map(FileObject::new)
            .collect();
        list_store_search.extend_from_slice(&objects);

        status_search.set_text(&format!(
            "{} matches found in {:.2} ms (scanned {} items in RAM)",
            objects.len(),
            (res.elapsed_micros as f64) / 1000.0,
            res.total_scanned
        ));
    });

    // Hook search triggers with 25ms debounce for silky-smooth typing
    let search_clone = Rc::clone(&perform_search);
    let debounce_source_id = Rc::new(RefCell::new(None::<glib::SourceId>));
    let debounce_clone = Rc::clone(&debounce_source_id);

    search_entry.connect_search_changed(move |entry| {
        let text = entry.text().to_string();
        let search_fn = Rc::clone(&search_clone);

        if let Some(source_id) = debounce_clone.borrow_mut().take() {
            source_id.remove();
        }

        let debounce_reset = Rc::clone(&debounce_clone);
        let id = glib::timeout_add_local_once(std::time::Duration::from_millis(25), move || {
            *debounce_reset.borrow_mut() = None;
            search_fn(text);
        });
        *debounce_clone.borrow_mut() = Some(id);
    });

    // Down Arrow in search entry jumps directly into the results list
    let col_view_for_search = column_view.clone();
    let search_key_ctrl = gtk4::EventControllerKey::new();
    search_key_ctrl.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == gtk4::gdk::Key::Down {
            col_view_for_search.grab_focus();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    search_entry.add_controller(search_key_ctrl);

    let search_clone2 = Rc::clone(&perform_search);
    let entry_clone2 = search_entry.clone();
    match_case_btn.connect_toggled(move |_| {
        search_clone2(entry_clone2.text().to_string());
    });

    let search_clone3 = Rc::clone(&perform_search);
    let entry_clone3 = search_entry.clone();
    regex_btn.connect_toggled(move |_| {
        search_clone3(entry_clone3.text().to_string());
    });

    // Hook category buttons
    for (btn, cat) in category_buttons {
        let cat_ref = Rc::clone(&active_category);
        let perform_ref = Rc::clone(&perform_search);
        let entry_ref = search_entry.clone();
        btn.connect_toggled(move |b| {
            if b.is_active() {
                *cat_ref.borrow_mut() = cat;
                perform_ref(entry_ref.text().to_string());
            }
        });
    }

    // Hook Rescan Button
    let db_rescan = Arc::clone(&db);
    let scan_path_rescan = scan_path.clone();
    let tx_rescan = tx.clone();
    let spinner_rescan = index_spinner.clone();
    let title_rescan = title_widget.clone();
    let status_rescan = status_label.clone();
    let trigger_rescan = Rc::new(move || {
        spinner_rescan.set_visible(true);
        spinner_rescan.start();
        title_rescan.set_subtitle("Re-indexing filesystem...");
        status_rescan.set_text("Scanning storage drives in parallel...");

        let db_thread = Arc::clone(&db_rescan);
        let path_thread = scan_path_rescan.clone();
        let tx_thread = tx_rescan.clone();
        std::thread::spawn(move || {
            let stats = db_thread.index_roots(&[path_thread]);
            let _ = tx_thread.send_blocking((stats.total_files + stats.total_dirs, stats.elapsed_ms));
        });
    });

    let rescan_clone = Rc::clone(&trigger_rescan);
    rescan_button.connect_clicked(move |_| {
        rescan_clone();
    });

    // Custom Compact About Dialog
    let window_for_about = window.clone();
    let total_items_about = Rc::clone(&current_total_items);
    btn_about.connect_clicked(move |_| {
        menu_popover.popdown();
        let total = *total_items_about.borrow();

        let about_win = gtk4::Window::builder()
            .transient_for(&window_for_about)
            .modal(true)
            .title("About Everything Linux")
            .default_width(380)
            .resizable(false)
            .css_classes(["about-window"])
            .build();

        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        vbox.add_css_class("about-card");

        // App Icon & Title
        let icon = gtk4::Image::from_icon_name("system-search");
        icon.set_pixel_size(44);
        vbox.append(&icon);

        let title_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        title_box.set_halign(gtk4::Align::Center);
        let title_lbl = gtk4::Label::builder()
            .label("Everything Linux")
            .css_classes(["about-app-title"])
            .build();
        let ver_lbl = gtk4::Label::builder()
            .label(format!("v{}", APP_VERSION))
            .css_classes(["about-version-tag"])
            .build();
        title_box.append(&title_lbl);
        title_box.append(&ver_lbl);
        vbox.append(&title_box);

        let desc_lbl = gtk4::Label::builder()
            .label("Instant in-memory desktop file search engine for Linux.")
            .css_classes(["dim-path"])
            .justify(gtk4::Justification::Center)
            .wrap(true)
            .build();
        vbox.append(&desc_lbl);

        // Developer Section Card
        let dev_card = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        dev_card.add_css_class("about-author-card");

        let dev_label_top = gtk4::Label::builder()
            .label("Created & Maintained by")
            .xalign(0.0)
            .css_classes(["dim-path"])
            .build();
        dev_card.append(&dev_label_top);

        let dev_name = gtk4::Label::builder()
            .label(AUTHOR_NAME)
            .xalign(0.0)
            .css_classes(["about-author-name"])
            .build();
        dev_card.append(&dev_name);

        let github_btn = gtk4::Button::builder()
            .label("github.com/arghadeeplaskar")
            .icon_name("web-browser-symbolic")
            .css_classes(["about-github-btn", "flat"])
            .build();
        github_btn.connect_clicked(|_| {
            let _ = open::that(AUTHOR_GITHUB);
        });
        dev_card.append(&github_btn);
        vbox.append(&dev_card);

        // Telemetry stats row
        let stats_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        stats_box.set_halign(gtk4::Align::Center);

        let s1 = gtk4::Label::builder()
            .label(format!("⚡ {} items in RAM", total))
            .css_classes(["about-stats-pill"])
            .build();
        let s2 = gtk4::Label::builder()
            .label(format!("💾 {:.1} MB", (total * 96) as f64 / 1_048_576.0))
            .css_classes(["about-stats-pill"])
            .build();
        let s3 = gtk4::Label::builder()
            .label("● inotify live")
            .css_classes(["about-stats-pill"])
            .build();
        stats_box.append(&s1);
        stats_box.append(&s2);
        stats_box.append(&s3);
        vbox.append(&stats_box);

        // Footer & Close Button
        let footer_lbl = gtk4::Label::builder()
            .label(format!("© 2026 {} • MIT License", AUTHOR_NAME))
            .css_classes(["about-footer"])
            .build();
        vbox.append(&footer_lbl);

        let close_btn = gtk4::Button::builder()
            .label("Close")
            .css_classes(["suggested-action"])
            .build();
        let win_close = about_win.clone();
        close_btn.connect_clicked(move |_| {
            win_close.close();
        });
        vbox.append(&close_btn);

        about_win.set_child(Some(&vbox));
        about_win.present();
    });

    // Shortcuts Window
    let window_for_shortcuts = window.clone();
    btn_shortcuts.connect_clicked(move |_| {
        let shortcuts_win = gtk4::ShortcutsWindow::builder()
            .transient_for(&window_for_shortcuts)
            .modal(true)
            .build();

        let section = gtk4::ShortcutsSection::builder().build();
        let group1 = gtk4::ShortcutsGroup::builder().title("Search & Navigation").build();

        let s1 = gtk4::ShortcutsShortcut::builder().title("Focus Search Entry").accelerator("<Primary>f").build();
        let s2 = gtk4::ShortcutsShortcut::builder().title("Clear Search Query").accelerator("Escape").build();
        let s3 = gtk4::ShortcutsShortcut::builder().title("Re-index Database").accelerator("F5").build();
        group1.append(&s1);
        group1.append(&s2);
        group1.append(&s3);

        let group2 = gtk4::ShortcutsGroup::builder().title("File Actions").build();
        let a1 = gtk4::ShortcutsShortcut::builder().title("Open Selected File").accelerator("Return").build();
        let a2 = gtk4::ShortcutsShortcut::builder().title("Open Containing Folder").accelerator("<Primary>Return").build();
        let a3 = gtk4::ShortcutsShortcut::builder().title("Copy Full Path").accelerator("<Primary>c").build();
        group2.append(&a1);
        group2.append(&a2);
        group2.append(&a3);

        section.append(&group1);
        section.append(&group2);
        shortcuts_win.set_child(Some(&section));
        shortcuts_win.present();
    });

    // Keyboard Controller
    let key_controller = gtk4::EventControllerKey::new();
    let search_for_keys = search_entry.clone();
    let rescan_for_keys = Rc::clone(&trigger_rescan);
    let sel_for_keys = selection_model.clone();
    let win_for_keys = window.clone();

    key_controller.connect_key_pressed(move |_, keyval, _keycode, state| {
        if keyval == gtk4::gdk::Key::F5 {
            rescan_for_keys();
            return glib::Propagation::Stop;
        }

        if state.contains(gtk4::gdk::ModifierType::CONTROL_MASK) && (keyval == gtk4::gdk::Key::f || keyval == gtk4::gdk::Key::F) {
            search_for_keys.grab_focus();
            search_for_keys.select_region(0, -1);
            return glib::Propagation::Stop;
        }

        if keyval == gtk4::gdk::Key::Escape {
            search_for_keys.set_text("");
            return glib::Propagation::Stop;
        }

        if state.contains(gtk4::gdk::ModifierType::CONTROL_MASK) && (keyval == gtk4::gdk::Key::c || keyval == gtk4::gdk::Key::C) {
            if let Some(selected_item) = sel_for_keys.selected_item() {
                if let Some(file_obj) = selected_item.downcast_ref::<FileObject>() {
                    if let Some(entry) = file_obj.entry() {
                        let display = gtk4::prelude::WidgetExt::display(&win_for_keys);
                        display.clipboard().set_text(&entry.full_path().to_string_lossy());
                        return glib::Propagation::Stop;
                    }
                }
            }
        }

        if state.contains(gtk4::gdk::ModifierType::CONTROL_MASK) && (keyval == gtk4::gdk::Key::Return || keyval == gtk4::gdk::Key::KP_Enter) {
            if let Some(selected_item) = sel_for_keys.selected_item() {
                if let Some(file_obj) = selected_item.downcast_ref::<FileObject>() {
                    if let Some(entry) = file_obj.entry() {
                        let _ = open::that(entry.parent);
                        return glib::Propagation::Stop;
                    }
                }
            }
        }

        glib::Propagation::Proceed
    });

    window.add_controller(key_controller);
}
