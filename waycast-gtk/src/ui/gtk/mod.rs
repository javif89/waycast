use gio::prelude::ApplicationExt;
use glib;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use gtk::{
    ApplicationWindow, Box as GtkBox, CellRendererPixbuf, CellRendererText, Entry,
    EventControllerKey, IconTheme, ListStore, Orientation, ScrolledWindow, TreePath, TreeView,
    TreeViewColumn,
};
use gtk4_layer_shell as layerShell;
use layerShell::LayerShell;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use waycast_core::WaycastLauncher;
use waycast_core::cache::CacheTTL;

// Column indices for the ListStore
const COL_ICON: u32 = 0;
const COL_TEXT: u32 = 1;
const COL_ID: u32 = 2;

pub struct GtkLauncherUI {
    window: ApplicationWindow,
    #[allow(dead_code)]
    tree_view: TreeView,
    #[allow(dead_code)]
    list_store: ListStore,
}

impl GtkLauncherUI {
    pub fn new(app: &gtk::Application, launcher: WaycastLauncher) -> Self {
        let launcher = Rc::new(std::cell::RefCell::new(launcher));
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Waycast")
            .default_width(800)
            .default_height(500)
            .resizable(false)
            .build();

        let main_box = GtkBox::new(Orientation::Vertical, 0);

        let search_input = Entry::new();
        search_input.set_placeholder_text(Some("Search..."));
        search_input.set_widget_name("search-input");
        search_input.add_css_class("launcher-search");

        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_min_content_height(300);

        // Create the list store with columns: Icon (Pixbuf), Text (String with markup), ID (String)
        let list_store = ListStore::new(&[
            Pixbuf::static_type(), // Icon
            String::static_type(), // Combined title/description with Pango markup
            String::static_type(), // Hidden ID
        ]);

        // Create the TreeView
        let tree_view = TreeView::with_model(&list_store);
        tree_view.set_headers_visible(false);
        tree_view.set_enable_search(false);
        tree_view.set_activate_on_single_click(false);
        tree_view.set_vexpand(true);
        tree_view.set_can_focus(true);
        tree_view.set_widget_name("results-list");
        tree_view.add_css_class("launcher-list");

        // Create icon column with CellRendererPixbuf
        let icon_renderer = CellRendererPixbuf::new();
        icon_renderer.set_padding(5, 5);
        let icon_column = TreeViewColumn::new();
        icon_column.pack_start(&icon_renderer, false);
        icon_column.add_attribute(&icon_renderer, "pixbuf", COL_ICON as i32);
        tree_view.append_column(&icon_column);

        // Create text column with CellRendererText using Pango markup
        let text_renderer = CellRendererText::new();
        text_renderer.set_ellipsize(gtk::pango::EllipsizeMode::End);
        text_renderer.set_padding(5, 5);
        let text_column = TreeViewColumn::new();
        text_column.set_expand(true);
        text_column.pack_start(&text_renderer, true);
        text_column.add_attribute(&text_renderer, "markup", COL_TEXT as i32);
        tree_view.append_column(&text_column);

        // Get the selection model
        let selection = tree_view.selection();
        selection.set_mode(gtk::SelectionMode::Single);

        scrolled_window.set_child(Some(&tree_view));
        scrolled_window.set_widget_name("results-container");
        scrolled_window.add_css_class("launcher-results-container");

        main_box.append(&search_input);
        main_box.append(&scrolled_window);
        main_box.set_widget_name("main-container");
        main_box.add_css_class("launcher-main");

        window.set_child(Some(&main_box));
        window.set_widget_name("launcher-window");
        window.add_css_class("launcher-window");

        // Set up layer shell so the launcher can float
        window.init_layer_shell();
        let edges = [
            layerShell::Edge::Top,
            layerShell::Edge::Bottom,
            layerShell::Edge::Left,
            layerShell::Edge::Right,
        ];
        for edge in edges {
            window.set_anchor(edge, false);
        }
        window.set_keyboard_mode(layerShell::KeyboardMode::OnDemand);
        window.set_layer(layerShell::Layer::Top);

        // Set initial focus to search input so user can start typing immediately
        search_input.grab_focus();

        // Helper function to populate the TreeView
        let populate_tree_view = |list_store: &ListStore,
                                  results: &[Box<dyn waycast_core::LauncherListItem>],
                                  icon_theme: &IconTheme| {
            list_store.clear();

            for entry in results.iter() {
                // Load icon as Pixbuf (with caching)
                let pixbuf =
                    if let Some(icon_path) = find_icon_file(&entry.icon(), "48", icon_theme) {
                        println!("Icon for: {} @ {}", entry.title(), icon_path.display());
                        Pixbuf::from_file_at_scale(&icon_path, 48, 48, true).ok()
                    } else {
                        None
                    }
                    .unwrap_or_else(|| {
                        // Try to get default icon from theme or create empty pixbuf
                        if let Some(default_path) =
                            find_icon_file("application-x-executable", "48", icon_theme)
                        {
                            Pixbuf::from_file_at_scale(&default_path, 48, 48, true).ok()
                        } else {
                            None
                        }
                        .unwrap_or_else(|| {
                            // Last resort: create an empty pixbuf
                            Pixbuf::new(gtk::gdk_pixbuf::Colorspace::Rgb, true, 8, 48, 48)
                                .unwrap_or_else(|| {
                                    Pixbuf::new(gtk::gdk_pixbuf::Colorspace::Rgb, true, 8, 1, 1)
                                        .unwrap()
                                })
                        })
                    });

                // Create Pango markup for title and description
                let text_markup = if let Some(desc) = entry.description() {
                    format!(
                        "<b>{}</b>\n<small><i>{}</i></small>",
                        glib::markup_escape_text(&entry.title()),
                        glib::markup_escape_text(&desc)
                    )
                } else {
                    format!("<b>{}</b>", glib::markup_escape_text(&entry.title()))
                };

                // Add row to ListStore
                let iter = list_store.append();
                list_store.set(
                    &iter,
                    &[
                        (COL_ICON, &pixbuf),
                        (COL_TEXT, &text_markup),
                        (COL_ID, &entry.id()),
                    ],
                );
            }
        };

        // Set up async search handlers to prevent UI blocking
        let launcher_for_search = launcher.clone();
        let list_store_for_search = list_store.clone();
        let tree_view_for_search = tree_view.clone();

        // Add debouncing to avoid excessive searches with generation counter
        let search_generation = Rc::new(RefCell::new(0u64));

        search_input.connect_changed(move |entry| {
            let query = entry.text().to_string();
            let display = gtk::gdk::Display::default().unwrap();
            let icon_theme = IconTheme::for_display(&display);

            // Increment generation to cancel any pending searches
            *search_generation.borrow_mut() += 1;

            if query.trim().is_empty() {
                // Handle empty query synchronously for immediate response
                let mut launcher_ref = launcher_for_search.borrow_mut();
                let results = launcher_ref.get_default_results();
                populate_tree_view(&list_store_for_search, &results, &icon_theme);
                drop(launcher_ref);

                // Select first item
                if let Some(iter) = list_store_for_search.iter_first() {
                    tree_view_for_search.selection().select_iter(&iter);
                }
            } else {
                // Debounced async search for non-empty queries
                let launcher_clone = launcher_for_search.clone();
                let list_store_clone = list_store_for_search.clone();
                let tree_view_clone = tree_view_for_search.clone();

                let current_generation = *search_generation.borrow();
                let generation_check = search_generation.clone();
                let _timeout_id =
                    glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
                        // Check if this search is still the current one
                        if *generation_check.borrow() != current_generation {
                            return glib::ControlFlow::Break; // This search was superseded
                        }

                        let launcher_clone = launcher_clone.clone();
                        let list_store_clone = list_store_clone.clone();
                        let tree_view_clone = tree_view_clone.clone();
                        let query = query.clone();
                        let icon_theme = icon_theme.clone();

                        glib::spawn_future_local(async move {
                            // Run search and populate immediately
                            let mut launcher_ref = launcher_clone.borrow_mut();
                            let results = launcher_ref.search(&query);
                            populate_tree_view(&list_store_clone, &results, &icon_theme);
                            drop(launcher_ref);

                            // Select first item
                            if let Some(iter) = list_store_clone.iter_first() {
                                tree_view_clone.selection().select_iter(&iter);
                            }
                        });

                        glib::ControlFlow::Break
                    });
            }
        });

        // Connect Enter key activation for search input
        let launcher_for_enter = launcher.clone();
        let list_store_for_enter = list_store.clone();
        let tree_view_for_enter = tree_view.clone();
        let app_for_enter = app.clone();
        search_input.connect_activate(move |_| {
            let (selected_paths, _) = tree_view_for_enter.selection().selected_rows();
            if let Some(path) = selected_paths.first() {
                if let Some(iter) = list_store_for_enter.iter(path) {
                    let id: String = list_store_for_enter.get(&iter, COL_ID as i32);
                    match launcher_for_enter.borrow().execute_item_by_id(&id) {
                        Ok(_) => app_for_enter.quit(),
                        Err(e) => eprintln!("Failed to launch app: {:?}", e),
                    }
                }
            }
        });

        // Add key handler for launcher-style navigation
        let search_key_controller = EventControllerKey::new();
        let tree_view_for_keys = tree_view.clone();
        let list_store_for_keys = list_store.clone();
        search_key_controller.connect_key_pressed(move |_controller, keyval, _keycode, _state| {
            match keyval {
                gtk::gdk::Key::Down => {
                    let selection = tree_view_for_keys.selection();
                    let (selected_paths, _) = selection.selected_rows();

                    if let Some(path) = selected_paths.first() {
                        let indices = path.indices();
                        if let Some(index) = indices.first() {
                            let next_index = index + 1;
                            if next_index < list_store_for_keys.iter_n_children(None) {
                                let next_path = gtk::TreePath::from_indices(&[next_index]);
                                selection.select_path(&next_path);
                                tree_view_for_keys.scroll_to_cell(
                                    Some(&next_path),
                                    None::<&TreeViewColumn>,
                                    false,
                                    0.0,
                                    0.0,
                                );
                            }
                        }
                    } else if list_store_for_keys.iter_n_children(None) > 0 {
                        // No selection, select first item
                        let first_path = gtk::TreePath::from_indices(&[0]);
                        selection.select_path(&first_path);
                        tree_view_for_keys.scroll_to_cell(
                            Some(&first_path),
                            None::<&TreeViewColumn>,
                            false,
                            0.0,
                            0.0,
                        );
                    }
                    gtk::glib::Propagation::Stop
                }
                gtk::gdk::Key::Up => {
                    let selection = tree_view_for_keys.selection();
                    let (selected_paths, _) = selection.selected_rows();

                    if let Some(path) = selected_paths.first() {
                        let indices = path.indices();
                        if let Some(index) = indices.first() {
                            if *index > 0 {
                                let prev_index = index - 1;
                                let prev_path = gtk::TreePath::from_indices(&[prev_index]);
                                selection.select_path(&prev_path);
                                tree_view_for_keys.scroll_to_cell(
                                    Some(&prev_path),
                                    None::<&TreeViewColumn>,
                                    false,
                                    0.0,
                                    0.0,
                                );
                            }
                        }
                    }
                    gtk::glib::Propagation::Stop
                }
                _ => gtk::glib::Propagation::Proceed,
            }
        });
        search_input.add_controller(search_key_controller);

        // Add ESC key handler at window level
        let window_key_controller = EventControllerKey::new();
        let app_for_esc = app.clone();
        window_key_controller.connect_key_pressed(move |_controller, keyval, _keycode, _state| {
            if keyval == gtk::gdk::Key::Escape {
                app_for_esc.quit();
                gtk::glib::Propagation::Stop
            } else {
                gtk::glib::Propagation::Proceed
            }
        });
        window.add_controller(window_key_controller);

        // Connect tree view row activation signal
        let launcher_for_activate = launcher.clone();
        let app_for_activate = app.clone();
        let list_store_for_activate = list_store.clone();
        tree_view.connect_row_activated(move |_, path, _| {
            if let Some(iter) = list_store_for_activate.iter(path) {
                let id: String = list_store_for_activate.get(&iter, COL_ID as i32);
                match launcher_for_activate.borrow().execute_item_by_id(&id) {
                    Ok(_) => app_for_activate.quit(),
                    Err(e) => eprintln!("Failed to launch app: {:?}", e),
                }
            }
        });

        // Initialize with default results
        let display = gtk::gdk::Display::default().unwrap();
        let icon_theme = IconTheme::for_display(&display);
        let mut launcher_ref = launcher.borrow_mut();
        let results = launcher_ref.get_default_results();
        populate_tree_view(&list_store, &results, &icon_theme);
        drop(launcher_ref); // Release the borrow

        // Select the first item if available
        if let Some(iter) = list_store.iter_first() {
            tree_view.selection().select_iter(&iter);
        }

        Self {
            window,
            tree_view,
            list_store,
        }
    }
}

impl GtkLauncherUI {
    pub fn show(&self) {
        self.window.present();
    }

    /// Apply default built-in CSS styles
    pub fn apply_default_css(&self) -> Result<(), String> {
        const DEFAULT_CSS: &str = include_str!("default.css");

        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_data(DEFAULT_CSS);

        if let Some(display) = gtk::gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &css_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
            Ok(())
        } else {
            Err("Could not get default display".to_string())
        }
    }

    pub fn apply_css<P: AsRef<Path>>(&self, css_path: P) -> Result<(), String> {
        let css_provider = gtk::CssProvider::new();

        // Check if file exists first
        if !css_path.as_ref().exists() {
            return Err(format!(
                "CSS file does not exist: {}",
                css_path.as_ref().display()
            ));
        }

        // Try to load the CSS file
        // Note: load_from_path doesn't return a Result, it panics on error
        // So we'll use a different approach with error handling
        use std::fs;
        match fs::read_to_string(css_path.as_ref()) {
            Ok(css_content) => {
                css_provider.load_from_data(&css_content);

                // Apply the CSS to the display
                if let Some(display) = gtk::gdk::Display::default() {
                    gtk::style_context_add_provider_for_display(
                        &display,
                        &css_provider,
                        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                    );
                    Ok(())
                } else {
                    Err("Could not get default display".to_string())
                }
            }
            Err(e) => Err(format!("Failed to read CSS file: {}", e)),
        }
    }
}

fn find_icon_file(
    icon_name: &str,
    size: &str,
    icon_theme: &IconTheme,
) -> Option<std::path::PathBuf> {
    let cache_key = format!("icon:{}:{}", icon_name, size);
    let cache = waycast_core::cache::get();

    let result = cache.remember_with_ttl(&cache_key, CacheTTL::hours(24), || {
        search_for_icon(icon_name, size, icon_theme)
    });

    if let Ok(opt_path) = result {
        return opt_path;
    }

    search_for_icon(icon_name, size, icon_theme)
}

fn search_for_icon(
    icon_name: &str,
    size: &str,
    icon_theme: &IconTheme,
) -> Option<std::path::PathBuf> {
    // Before doing everything below, check if it's a path
    if Path::new(icon_name).exists() {
        return Some(PathBuf::from(icon_name));
    }

    let pixmap_paths: Vec<PathBuf> = icon_theme
        .search_path()
        .into_iter()
        .filter(|p| p.to_string_lossy().contains("pixmap"))
        .collect();
    let search_paths: Vec<PathBuf> = icon_theme
        .search_path()
        .into_iter()
        .filter(|p| p.to_string_lossy().contains("icons"))
        .collect();

    let sizes = [size, "scalable"];
    let categories = ["apps", "applications", "mimetypes"];
    let extensions = ["svg", "png", "xpm"];

    // Build the search paths
    let mut search_in: Vec<PathBuf> = Vec::new();
    // Do all the theme directories first and high color second
    for base in &search_paths {
        for size in sizes {
            for cat in &categories {
                let path = base
                    .join(icon_theme.theme_name())
                    .join(if size != "scalable" {
                        format!("{}x{}", size, size)
                    } else {
                        size.to_string()
                    })
                    .join(cat);

                if path.exists() {
                    search_in.push(path);
                }
            }
        }
    }

    for base in &search_paths {
        for size in sizes {
            for cat in &categories {
                let path = base
                    .join("hicolor")
                    .join(if size != "scalable" {
                        format!("{}x{}", size, size)
                    } else {
                        size.to_string()
                    })
                    .join(cat);

                if path.exists() {
                    search_in.push(path);
                }
            }
        }
    }
    // Last resort, search pixmaps directly (no subdirectories)
    for base in &pixmap_paths {
        if !base.exists() {
            continue;
        }

        for ext in &extensions {
            let direct_icon = base.join(format!("{}.{}", icon_name, ext));
            if direct_icon.exists() {
                return Some(direct_icon);
            }
        }
    }

    for s in &search_in {
        for ext in &extensions {
            let icon_path = s.join(format!("{}.{}", icon_name, ext));
            if icon_path.exists() {
                return Some(icon_path);
            }
        }
    }

    None
}
