use gio::prelude::ApplicationExt;
use glib;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use gtk::{
    ApplicationWindow, Box as GtkBox, CellRendererPixbuf, CellRendererText, Entry,
    EventControllerKey, ListStore, Orientation, ScrolledWindow, TreeView,
    TreeViewColumn,
};
use gtk4_layer_shell as layerShell;
use layerShell::LayerShell;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;
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
        let ui_start = Instant::now();
        eprintln!("[PROFILE] Starting UI creation");
        
        let launcher = Rc::new(std::cell::RefCell::new(launcher));
        
        let window_start = Instant::now();
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Waycast")
            .default_width(800)
            .default_height(500)
            .resizable(false)
            .build();
        eprintln!("[PROFILE] Window creation: {:?}", window_start.elapsed());

        let widgets_start = Instant::now();
        let main_box = GtkBox::new(Orientation::Vertical, 0);

        let search_input = Entry::new();
        search_input.set_placeholder_text(Some("Search..."));
        search_input.set_widget_name("search-input");
        search_input.add_css_class("launcher-search");

        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_min_content_height(300);

        // Create the list store with columns: Icon (Pixbuf), Text (String with markup), ID (String)
        let list_store_start = Instant::now();
        let list_store = ListStore::new(&[
            Pixbuf::static_type(), // Icon
            String::static_type(), // Combined title/description with Pango markup
            String::static_type(), // Hidden ID
        ]);
        eprintln!("[PROFILE] ListStore creation: {:?}", list_store_start.elapsed());

        // Create the TreeView
        let tree_view_start = Instant::now();
        let tree_view = TreeView::with_model(&list_store);
        tree_view.set_headers_visible(false);
        tree_view.set_enable_search(false);
        tree_view.set_activate_on_single_click(false);
        tree_view.set_vexpand(true);
        tree_view.set_can_focus(true);
        tree_view.set_widget_name("results-list");
        tree_view.add_css_class("launcher-list");
        eprintln!("[PROFILE] TreeView creation: {:?}", tree_view_start.elapsed());

        // Create icon column with CellRendererPixbuf
        let columns_start = Instant::now();
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
        eprintln!("[PROFILE] TreeView columns setup: {:?}", columns_start.elapsed());

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
        eprintln!("[PROFILE] Basic widgets creation: {:?}", widgets_start.elapsed());

        window.set_child(Some(&main_box));
        window.set_widget_name("launcher-window");
        window.add_css_class("launcher-window");

        // Set up layer shell so the launcher can float
        let layer_shell_start = Instant::now();
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
        eprintln!("[PROFILE] Layer shell setup: {:?}", layer_shell_start.elapsed());

        // Set initial focus to search input so user can start typing immediately
        search_input.grab_focus();

        // Helper function to populate the TreeView
        let populate_tree_view =
            |list_store: &ListStore, results: &[Box<dyn waycast_core::LauncherListItem>]| {
                let populate_start = Instant::now();
                eprintln!("[PROFILE] Starting to populate TreeView with {} items", results.len());
                
                let clear_start = Instant::now();
                list_store.clear();
                eprintln!("[PROFILE]   ListStore clear: {:?}", clear_start.elapsed());

                let mut total_icon_time = std::time::Duration::new(0, 0);
                let mut total_markup_time = std::time::Duration::new(0, 0);
                let mut total_insert_time = std::time::Duration::new(0, 0);

                for (idx, entry) in results.iter().enumerate() {
                    // Load icon as Pixbuf (with caching)
                    let icon_start = Instant::now();
                    let pixbuf = if let Some(icon_path) = find_icon_file(&entry.icon(), "48") {
                        let pixbuf_load_start = Instant::now();
                        let pixbuf = Pixbuf::from_file_at_scale(&icon_path, 48, 48, true).ok();
                        if idx < 5 {
                            eprintln!("[PROFILE]     Pixbuf::from_file_at_scale: {:?}", pixbuf_load_start.elapsed());
                        }
                        pixbuf
                    } else {
                        None
                    }
                    .unwrap_or_else(|| {
                        // Try to get default icon from theme or create empty pixbuf
                        if let Some(default_path) = find_icon_file("application-x-executable", "48")
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
                    let icon_elapsed = icon_start.elapsed();
                    total_icon_time += icon_elapsed;
                    if idx < 5 || idx % 50 == 0 {
                        eprintln!("[PROFILE]   Icon {} load: {:?}", idx, icon_elapsed);
                    }

                    // Create Pango markup for title and description
                    let markup_start = Instant::now();
                    let text_markup = if let Some(desc) = entry.description() {
                        format!(
                            "<b>{}</b>\n<small><i>{}</i></small>",
                            glib::markup_escape_text(&entry.title()),
                            glib::markup_escape_text(&desc)
                        )
                    } else {
                        format!("<b>{}</b>", glib::markup_escape_text(&entry.title()))
                    };
                    let markup_elapsed = markup_start.elapsed();
                    total_markup_time += markup_elapsed;

                    // Add row to ListStore
                    let insert_start = Instant::now();
                    let iter = list_store.append();
                    list_store.set(
                        &iter,
                        &[
                            (COL_ICON, &pixbuf),
                            (COL_TEXT, &text_markup),
                            (COL_ID, &entry.id()),
                        ],
                    );
                    let insert_elapsed = insert_start.elapsed();
                    total_insert_time += insert_elapsed;
                }
                
                eprintln!("[PROFILE] TreeView population complete: {:?}", populate_start.elapsed());
                eprintln!("[PROFILE]   Total icon time: {:?} (avg: {:?})", 
                    total_icon_time, 
                    total_icon_time / results.len().max(1) as u32);
                eprintln!("[PROFILE]   Total markup time: {:?} (avg: {:?})", 
                    total_markup_time,
                    total_markup_time / results.len().max(1) as u32);
                eprintln!("[PROFILE]   Total insert time: {:?} (avg: {:?})", 
                    total_insert_time,
                    total_insert_time / results.len().max(1) as u32);
            };

        // Set up async search handlers to prevent UI blocking
        let launcher_for_search = launcher.clone();
        let list_store_for_search = list_store.clone();
        let tree_view_for_search = tree_view.clone();

        // Add debouncing to avoid excessive searches with generation counter
        let search_generation = Rc::new(RefCell::new(0u64));

        search_input.connect_changed(move |entry| {
            let search_start = Instant::now();
            let query = entry.text().to_string();
            eprintln!("[PROFILE] Search triggered for: '{}'", query);
            let _display = gtk::gdk::Display::default().unwrap();

            // Increment generation to cancel any pending searches
            *search_generation.borrow_mut() += 1;

            if query.trim().is_empty() {
                // Handle empty query synchronously for immediate response
                let default_start = Instant::now();
                let mut launcher_ref = launcher_for_search.borrow_mut();
                let results = launcher_ref.get_default_results();
                eprintln!("[PROFILE]   Get default results: {:?}", default_start.elapsed());
                populate_tree_view(&list_store_for_search, &results);
                drop(launcher_ref);

                // Select first item
                if let Some(iter) = list_store_for_search.iter_first() {
                    tree_view_for_search.selection().select_iter(&iter);
                }
                eprintln!("[PROFILE] Empty query handling total: {:?}", search_start.elapsed());
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

                        glib::spawn_future_local(async move {
                            // Run search and populate immediately
                            let mut launcher_ref = launcher_clone.borrow_mut();
                            let results = launcher_ref.search(&query);
                            populate_tree_view(&list_store_clone, &results);
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
            let activate_start = Instant::now();
            let (selected_paths, _) = tree_view_for_enter.selection().selected_rows();
            if let Some(path) = selected_paths.first() {
                if let Some(iter) = list_store_for_enter.iter(path) {
                    let id: String = list_store_for_enter.get(&iter, COL_ID as i32);
                    let execute_start = Instant::now();
                    match launcher_for_enter.borrow().execute_item_by_id(&id) {
                        Ok(_) => {
                            eprintln!("[PROFILE] Execute item: {:?}", execute_start.elapsed());
                            app_for_enter.quit();
                        },
                        Err(e) => eprintln!("Failed to launch app: {:?}", e),
                    }
                }
            }
            eprintln!("[PROFILE] Total activation handling: {:?}", activate_start.elapsed());
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
            let row_activate_start = Instant::now();
            if let Some(iter) = list_store_for_activate.iter(path) {
                let id: String = list_store_for_activate.get(&iter, COL_ID as i32);
                let execute_start = Instant::now();
                match launcher_for_activate.borrow().execute_item_by_id(&id) {
                    Ok(_) => {
                        eprintln!("[PROFILE] Row activation execute: {:?}", execute_start.elapsed());
                        app_for_activate.quit();
                    },
                    Err(e) => eprintln!("Failed to launch app: {:?}", e),
                }
            }
            eprintln!("[PROFILE] Row activation total: {:?}", row_activate_start.elapsed());
        });

        // Don't populate initially - defer until after window is shown
        eprintln!("[PROFILE] Total UI creation time: {:?}", ui_start.elapsed());
        eprintln!("[PROFILE] =======================================\n");
        
        // Store launcher for deferred population
        let launcher_for_defer = launcher.clone();
        let list_store_for_defer = list_store.clone();
        let tree_view_for_defer = tree_view.clone();
        
        // Schedule population after window is shown (like wofi does)
        glib::idle_add_local(move || {
            let defer_start = Instant::now();
            let mut launcher_ref = launcher_for_defer.borrow_mut();
            let results = launcher_ref.get_default_results();
            eprintln!("[PROFILE] Deferred: Get default results: {:?}", defer_start.elapsed());
            populate_tree_view(&list_store_for_defer, &results);
            drop(launcher_ref);
            
            // Select the first item if available
            if let Some(iter) = list_store_for_defer.iter_first() {
                tree_view_for_defer.selection().select_iter(&iter);
            }
            eprintln!("[PROFILE] Deferred: Population complete: {:?}", defer_start.elapsed());
            
            glib::ControlFlow::Break
        });
        
        Self {
            window,
            tree_view,
            list_store,
        }
    }
}

impl GtkLauncherUI {
    pub fn show(&self) {
        let show_start = Instant::now();
        
        // Try to minimize GTK's icon processing by showing window first without content
        let realize_start = Instant::now();
        gtk::prelude::WidgetExt::realize(&self.window); // Create the GDK resources
        eprintln!("[PROFILE] Window realize: {:?}", realize_start.elapsed());
        
        let show_window_start = Instant::now();
        self.window.show(); // Show without focusing
        eprintln!("[PROFILE] Window show: {:?}", show_window_start.elapsed());
        
        let present_start = Instant::now();
        self.window.present(); // Now present (focus)
        eprintln!("[PROFILE] Window present: {:?}", present_start.elapsed());
        
        eprintln!("[PROFILE] Total window display: {:?}", show_start.elapsed());
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

fn find_icon_file(icon_name: &str, size: &str) -> Option<std::path::PathBuf> {
    let icon_lookup_start = Instant::now();
    let cache_key = format!("icon:{}:{}", icon_name, size);
    let cache = waycast_core::cache::get();

    let _cache_start = Instant::now();
    let result = cache.remember_with_ttl(&cache_key, CacheTTL::hours(24), || {
        let freedesktop_start = Instant::now();
        let icon_result = freedesktop::get_icon(icon_name);
        eprintln!("[PROFILE]     freedesktop::get_icon('{}') uncached: {:?}", icon_name, freedesktop_start.elapsed());
        icon_result
    });
    
    let cached = result.is_ok();
    eprintln!("[PROFILE]     Icon lookup '{}' (cached={}): {:?}", icon_name, cached, icon_lookup_start.elapsed());

    if let Ok(opt_path) = result {
        return opt_path;
    }

    freedesktop::get_icon(icon_name)
}
