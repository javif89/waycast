use gio::prelude::ApplicationExt;
use glib;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use gtk::{
    ApplicationWindow, Box as GtkBox, Entry, EventControllerKey, FlowBox, FlowBoxChild, Image, Label,
    Orientation, ScrolledWindow,
};
use gtk4_layer_shell as layerShell;
use layerShell::LayerShell;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::time::Instant;
use waycast_core::WaycastLauncher;
use waycast_core::cache::CacheTTL;

pub struct GtkLauncherUI {
    window: ApplicationWindow,
    flow_box: FlowBox,
}

// Store item data in a simple struct
#[derive(Clone)]
pub struct FlowBoxItem {
    id: String,
    title: String,
    description: Option<String>,
    icon: String,
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
        scrolled_window.set_vexpand(true); // Make sure scroll window expands

        // Create FlowBox like wofi (much simpler than TreeView)
        let flow_box_start = Instant::now();
        let flow_box = FlowBox::new();
        flow_box.set_max_children_per_line(1); // Single column like a list
        flow_box.set_selection_mode(gtk::SelectionMode::Browse);
        flow_box.set_activate_on_single_click(false);
        flow_box.set_can_focus(true);
        flow_box.set_vexpand(true); // Expand vertically to fill space
        flow_box.set_hexpand(true); // Expand horizontally too
        flow_box.set_widget_name("results-list");
        flow_box.add_css_class("launcher-list");
        eprintln!("[PROFILE] FlowBox creation: {:?}", flow_box_start.elapsed());

        scrolled_window.set_child(Some(&flow_box));
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

        // Helper function to populate the FlowBox with direct GtkImage widgets
        let populate_flow_box =
            |flow_box: &FlowBox, results: &[Box<dyn waycast_core::LauncherListItem>]| {
                let populate_start = Instant::now();
                eprintln!("[PROFILE] Starting to populate FlowBox with {} items", results.len());
                
                let clear_start = Instant::now();
                // Remove all existing children
                while let Some(child) = flow_box.first_child() {
                    flow_box.remove(&child);
                }
                eprintln!("[PROFILE]   FlowBox clear: {:?}", clear_start.elapsed());

                let mut total_icon_time = std::time::Duration::new(0, 0);
                let mut total_widget_time = std::time::Duration::new(0, 0);

                for (idx, entry) in results.iter().enumerate() {
                    let widget_start = Instant::now();
                    
                    // Create main horizontal box for this item (like wofi does)
                    let item_box = GtkBox::new(Orientation::Horizontal, 8);
                    item_box.set_margin_start(8);
                    item_box.set_margin_end(8);
                    item_box.set_margin_top(4);
                    item_box.set_margin_bottom(4);

                    // Load icon and create GtkImage directly
                    let icon_start = Instant::now();
                    let image = if let Some(icon_path) = find_icon_file(&entry.icon(), "48") {
                        let image_load_start = Instant::now();
                        let image = Image::from_file(&icon_path);
                        image.set_pixel_size(48);
                        if idx < 5 {
                            eprintln!("[PROFILE]     Image::from_file: {:?}", image_load_start.elapsed());
                        }
                        image
                    } else {
                        // Fallback to default icon
                        if let Some(default_path) = find_icon_file("application-x-executable", "48") {
                            let image = Image::from_file(&default_path);
                            image.set_pixel_size(48);
                            image
                        } else {
                            // Last resort: empty image
                            let image = Image::new();
                            image.set_pixel_size(48);
                            image
                        }
                    };
                    let icon_elapsed = icon_start.elapsed();
                    total_icon_time += icon_elapsed;
                    if idx < 5 || idx % 50 == 0 {
                        eprintln!("[PROFILE]   Icon {} load: {:?}", idx, icon_elapsed);
                    }

                    // Create text label with markup
                    let label = Label::new(None);
                    let markup = if let Some(desc) = entry.description() {
                        format!(
                            "<b>{}</b>\n<small><i>{}</i></small>",
                            glib::markup_escape_text(&entry.title()),
                            glib::markup_escape_text(&desc)
                        )
                    } else {
                        format!("<b>{}</b>", glib::markup_escape_text(&entry.title()))
                    };
                    label.set_markup(&markup);
                    label.set_halign(gtk::Align::Start);
                    label.set_valign(gtk::Align::Center);
                    label.set_ellipsize(gtk::pango::EllipsizeMode::End);

                    // Pack into horizontal box
                    item_box.append(&image);
                    item_box.append(&label);

                    // Store the entry ID as widget name (simpler approach)
                    item_box.set_widget_name(&entry.id());

                    // Add to flow box
                    flow_box.insert(&item_box, -1);
                    
                    let widget_elapsed = widget_start.elapsed();
                    total_widget_time += widget_elapsed;
                }
                
                eprintln!("[PROFILE] FlowBox population complete: {:?}", populate_start.elapsed());
                eprintln!("[PROFILE]   Total icon time: {:?} (avg: {:?})", 
                    total_icon_time, 
                    total_icon_time / results.len().max(1) as u32);
                eprintln!("[PROFILE]   Total widget time: {:?} (avg: {:?})", 
                    total_widget_time,
                    total_widget_time / results.len().max(1) as u32);
            };

        // Set up async search handlers to prevent UI blocking
        let launcher_for_search = launcher.clone();
        let flow_box_for_search = flow_box.clone();

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
                populate_flow_box(&flow_box_for_search, &results);
                drop(launcher_ref);

                // Select first item
                if let Some(first_child) = flow_box_for_search.first_child() {
                    flow_box_for_search.select_child(&first_child.downcast::<FlowBoxChild>().unwrap());
                }
                eprintln!("[PROFILE] Empty query handling total: {:?}", search_start.elapsed());
            } else {
                // Debounced async search for non-empty queries
                let launcher_clone = launcher_for_search.clone();
                let flow_box_clone = flow_box_for_search.clone();

                let current_generation = *search_generation.borrow();
                let generation_check = search_generation.clone();
                let _timeout_id =
                    glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
                        // Check if this search is still the current one
                        if *generation_check.borrow() != current_generation {
                            return glib::ControlFlow::Break; // This search was superseded
                        }

                        let launcher_clone = launcher_clone.clone();
                        let flow_box_clone = flow_box_clone.clone();
                        let query = query.clone();

                        glib::spawn_future_local(async move {
                            // Run search and populate immediately
                            let search_exec_start = Instant::now();
                            let mut launcher_ref = launcher_clone.borrow_mut();
                            let results = launcher_ref.search(&query);
                            eprintln!("[PROFILE]   Search execution for '{}': {:?}", query, search_exec_start.elapsed());
                            populate_flow_box(&flow_box_clone, &results);
                            drop(launcher_ref);

                            // Select first item
                            if let Some(first_child) = flow_box_clone.first_child() {
                                flow_box_clone.select_child(&first_child.downcast::<FlowBoxChild>().unwrap());
                            }
                            eprintln!("[PROFILE] Search + populate total for '{}': {:?}", query, search_exec_start.elapsed());
                        });

                        glib::ControlFlow::Break
                    });
            }
        });

        // Connect Enter key activation for search input
        let launcher_for_enter = launcher.clone();
        let flow_box_for_enter = flow_box.clone();
        let app_for_enter = app.clone();
        search_input.connect_activate(move |_| {
            let activate_start = Instant::now();
            if let Some(selected_child) = flow_box_for_enter.selected_children().first() {
                if let Some(item_box) = selected_child.child() {
                    let id = item_box.widget_name();
                    let execute_start = Instant::now();
                    match launcher_for_enter.borrow().execute_item_by_id(id.as_str()) {
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
        let flow_box_for_keys = flow_box.clone();
        let scrolled_window_for_keys = scrolled_window.clone();
        search_key_controller.connect_key_pressed(move |_controller, keyval, _keycode, _state| {
            let key_start = Instant::now();
            
            // Helper function to scroll to the selected widget
            let scroll_to_selected = || {
                if let Some(selected_child) = flow_box_for_keys.selected_children().first() {
                    // Get the widget's allocation to determine scroll position
                    let allocation = selected_child.allocation();
                    let _scroll_allocation = scrolled_window_for_keys.allocation();
                    
                    // Get current scroll position
                    let vadjustment = scrolled_window_for_keys.vadjustment();
                    let current_scroll = vadjustment.value();
                    let page_size = vadjustment.page_size();
                    
                    // Calculate if we need to scroll
                    let widget_top = allocation.y() as f64;
                    let widget_bottom = (allocation.y() + allocation.height()) as f64;
                    let visible_top = current_scroll;
                    let visible_bottom = current_scroll + page_size;
                    
                    // Scroll if widget is not fully visible
                    if widget_top < visible_top {
                        // Widget is above visible area, scroll up
                        vadjustment.set_value(widget_top);
                    } else if widget_bottom > visible_bottom {
                        // Widget is below visible area, scroll down
                        vadjustment.set_value(widget_bottom - page_size);
                    }
                }
            };
            
            let result = match keyval {
                gtk::gdk::Key::Down => {
                    // Move to next item in FlowBox
                    if let Some(selected_children) = flow_box_for_keys.selected_children().first() {
                        if let Some(next_child) = selected_children.next_sibling() {
                            flow_box_for_keys.unselect_all();
                            flow_box_for_keys.select_child(&next_child.downcast::<FlowBoxChild>().unwrap());
                            scroll_to_selected();
                        }
                    } else if let Some(first_child) = flow_box_for_keys.first_child() {
                        // No selection, select first item
                        flow_box_for_keys.select_child(&first_child.downcast::<FlowBoxChild>().unwrap());
                        scroll_to_selected();
                    }
                    gtk::glib::Propagation::Stop
                }
                gtk::gdk::Key::Up => {
                    // Move to previous item in FlowBox
                    if let Some(selected_children) = flow_box_for_keys.selected_children().first() {
                        if let Some(prev_child) = selected_children.prev_sibling() {
                            flow_box_for_keys.unselect_all();
                            flow_box_for_keys.select_child(&prev_child.downcast::<FlowBoxChild>().unwrap());
                            scroll_to_selected();
                        }
                    }
                    gtk::glib::Propagation::Stop
                }
                _ => gtk::glib::Propagation::Proceed,
            };
            if matches!(keyval, gtk::gdk::Key::Down | gtk::gdk::Key::Up) {
                eprintln!("[PROFILE] Key navigation: {:?}", key_start.elapsed());
            }
            result
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

        // Connect flow box activation signal
        let launcher_for_activate = launcher.clone();
        let app_for_activate = app.clone();
        flow_box.connect_child_activated(move |_, child| {
            let row_activate_start = Instant::now();
            if let Some(item_box) = child.child() {
                let id = item_box.widget_name();
                let execute_start = Instant::now();
                match launcher_for_activate.borrow().execute_item_by_id(id.as_str()) {
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
        let flow_box_for_defer = flow_box.clone();
        
        // Schedule population after window is shown (like wofi does)
        glib::idle_add_local(move || {
            let defer_start = Instant::now();
            let mut launcher_ref = launcher_for_defer.borrow_mut();
            let results = launcher_ref.get_default_results();
            eprintln!("[PROFILE] Deferred: Get default results: {:?}", defer_start.elapsed());
            populate_flow_box(&flow_box_for_defer, &results);
            drop(launcher_ref);
            
            // Select the first item if available
            if let Some(first_child) = flow_box_for_defer.first_child() {
                flow_box_for_defer.select_child(&first_child.downcast::<FlowBoxChild>().unwrap());
            }
            eprintln!("[PROFILE] Deferred: Population complete: {:?}", defer_start.elapsed());
            
            glib::ControlFlow::Break
        });
        
        Self {
            window,
            flow_box,
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
