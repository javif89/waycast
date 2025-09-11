use gio::prelude::ApplicationExt;
use gio::ListStore;
use gtk::gdk::Texture;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{
    ApplicationWindow, Box as GtkBox, Entry, EventControllerKey, IconTheme, Image, Label, ListView,
    Orientation, ScrolledWindow, SignalListItemFactory, SingleSelection,
};
use gtk4_layer_shell as layerShell;
use layerShell::LayerShell;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use waycast_core::cache::CacheTTL;
use waycast_core::WaycastLauncher;

// GObject wrapper to store LauncherListItem in GTK's model system
mod imp {
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct LauncherItemObject {
        pub title: RefCell<String>,
        pub description: RefCell<Option<String>>,
        pub icon: RefCell<String>,
        pub id: RefCell<String>, // Store id to access original entry
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LauncherItemObject {
        const NAME: &'static str = "WaycastLauncherItemObject";
        type Type = super::LauncherItemObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for LauncherItemObject {}
}

glib::wrapper! {
    pub struct LauncherItemObject(ObjectSubclass<imp::LauncherItemObject>);
}

impl LauncherItemObject {
    pub fn new(title: String, description: Option<String>, icon: String, id: String) -> Self {
        let obj: Self = glib::Object::new();
        let imp = obj.imp();

        // Store the data
        *imp.title.borrow_mut() = title;
        *imp.description.borrow_mut() = description;
        *imp.icon.borrow_mut() = icon;
        *imp.id.borrow_mut() = id;

        obj
    }

    pub fn title(&self) -> String {
        self.imp().title.borrow().clone()
    }

    pub fn icon(&self) -> String {
        self.imp().icon.borrow().clone()
    }

    pub fn description(&self) -> Option<String> {
        self.imp().description.borrow().clone()
    }

    pub fn id(&self) -> String {
        self.imp().id.borrow().clone()
    }
}

pub struct GtkLauncherUI {
    window: ApplicationWindow,
}

impl GtkLauncherUI {
    pub fn new(app: &gtk::Application, launcher: WaycastLauncher) -> Self {
        let launcher = Rc::new(RefCell::new(launcher));
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

        // Create the list store and selection model
        let list_store = ListStore::new::<LauncherItemObject>();
        let selection = SingleSelection::new(Some(list_store.clone()));

        // Create factory for rendering list items
        let factory = SignalListItemFactory::new();

        // Setup factory to create widgets
        factory.connect_setup(move |_, list_item| {
            let container = GtkBox::new(Orientation::Horizontal, 10);
            container.set_widget_name("list-item");
            container.add_css_class("launcher-item");
            list_item.set_child(Some(&container));
        });

        // Setup factory to bind data to widgets
        factory.connect_bind(move |_, list_item| {
            let child = list_item.child().and_downcast::<GtkBox>().unwrap();

            // Clear existing children
            while let Some(first_child) = child.first_child() {
                child.remove(&first_child);
            }

            if let Some(item_obj) = list_item.item().and_downcast::<LauncherItemObject>() {
                let display = gtk::gdk::Display::default().unwrap();
                let icon_theme = gtk::IconTheme::for_display(&display);
                let icon_size = 48;

                // Create icon
                let image: gtk::Image;
                if let Some(icon_path) = find_icon_file(&item_obj.icon(), "48", &icon_theme) {
                    image = match Pixbuf::from_file_at_scale(icon_path, icon_size, icon_size, true)
                    {
                        Ok(pb) => {
                            let tex = Texture::for_pixbuf(&pb);
                            gtk::Image::from_paintable(Some(&tex))
                        }
                        Err(e) => {
                            eprintln!("err: {}", e);
                            Image::from_icon_name("application-x-executable")
                        }
                    }
                } else if let Some(default) = find_icon_file("vscode", "48", &icon_theme) {
                    image = gtk::Image::from_file(default);
                } else {
                    image = Image::from_icon_name("application-x-executable");
                }
                image.set_pixel_size(icon_size);
                image.set_widget_name("item-icon");
                image.add_css_class("launcher-icon");

                // Create text container (vertical box for title + description)
                let text_box = GtkBox::new(Orientation::Vertical, 2);
                text_box.set_hexpand(true);
                text_box.set_valign(gtk::Align::Center);
                text_box.set_widget_name("item-text");
                text_box.add_css_class("launcher-text");

                // Create title label
                let title_label = Label::new(Some(&item_obj.title()));
                title_label.set_xalign(0.0);
                title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                title_label.set_widget_name("item-title");
                title_label.add_css_class("launcher-title");
                text_box.append(&title_label);

                // Create description label if description exists
                if let Some(description) = item_obj.description() {
                    let desc_label = Label::new(Some(&description));
                    desc_label.set_xalign(0.0);
                    desc_label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
                    desc_label.set_widget_name("item-description");
                    desc_label.add_css_class("launcher-description");
                    desc_label.set_opacity(0.7);
                    text_box.append(&desc_label);
                }

                child.append(&image);
                child.append(&text_box);
            }
        });

        let list_view = ListView::new(Some(selection.clone()), Some(factory));
        list_view.set_vexpand(true);
        list_view.set_can_focus(true);
        list_view.set_widget_name("results-list");
        list_view.add_css_class("launcher-list");

        scrolled_window.set_child(Some(&list_view));
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

        // Set up async search handlers to prevent UI blocking
        let launcher_for_search = launcher.clone();
        let list_store_for_search = list_store.clone();
        let selection_for_search = selection.clone();

        // Add debouncing to avoid excessive searches with generation counter
        let search_generation = Rc::new(RefCell::new(0u64));

        search_input.connect_changed(move |entry| {
            let query = entry.text().to_string();

            // Increment generation to cancel any pending searches
            *search_generation.borrow_mut() += 1;

            if query.trim().is_empty() {
                // Handle empty query synchronously for immediate response
                let mut launcher_ref = launcher_for_search.borrow_mut();
                let results = launcher_ref.get_default_results();

                list_store_for_search.remove_all();
                for entry in results.iter() {
                    let item_obj = LauncherItemObject::new(
                        entry.title(),
                        entry.description(),
                        entry.icon(),
                        entry.id(),
                    );
                    list_store_for_search.append(&item_obj);
                }

                // Select first item
                if list_store_for_search.n_items() > 0 {
                    selection_for_search.set_selected(0);
                }
            } else {
                // Debounced async search for non-empty queries
                let launcher_clone = launcher_for_search.clone();
                let list_store_clone = list_store_for_search.clone();
                let selection_clone = selection_for_search.clone();

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
                        let selection_clone = selection_clone.clone();
                        let query = query.clone();

                        glib::spawn_future_local(async move {
                            // Run search and collect items immediately
                            let items: Vec<LauncherItemObject> = {
                                let mut launcher_ref = launcher_clone.borrow_mut();
                                let results = launcher_ref.search(&query);
                                results
                                    .iter()
                                    .map(|entry| {
                                        LauncherItemObject::new(
                                            entry.title(),
                                            entry.description(),
                                            entry.icon(),
                                            entry.id(),
                                        )
                                    })
                                    .collect()
                            };

                            // Update UI on main thread
                            list_store_clone.remove_all();
                            for item_obj in items {
                                list_store_clone.append(&item_obj);
                            }

                            // Select first item
                            if list_store_clone.n_items() > 0 {
                                selection_clone.set_selected(0);
                            }
                        });

                        glib::ControlFlow::Break
                    });
            }
        });

        // Connect Enter key activation for search input
        let launcher_for_enter = launcher.clone();
        let selection_for_enter = selection.clone();
        let app_for_enter = app.clone();
        search_input.connect_activate(move |_| {
            if let Some(selected_item) = selection_for_enter.selected_item() {
                if let Some(item_obj) = selected_item.downcast_ref::<LauncherItemObject>() {
                    let id = item_obj.id();
                    match launcher_for_enter.borrow().execute_item_by_id(&id) {
                        Ok(_) => app_for_enter.quit(),
                        Err(e) => eprintln!("Failed to launch app: {:?}", e),
                    }
                }
            }
        });

        // Add key handler for launcher-style navigation
        let search_key_controller = EventControllerKey::new();
        let selection_for_keys = selection.clone();
        search_key_controller.connect_key_pressed(move |_controller, keyval, _keycode, _state| {
            match keyval {
                gtk::gdk::Key::Down => {
                    let current_pos = selection_for_keys.selected();
                    let n_items = selection_for_keys.model().unwrap().n_items();
                    if current_pos < n_items - 1 {
                        selection_for_keys.set_selected(current_pos + 1);
                    } else if n_items > 0 && current_pos == gtk::INVALID_LIST_POSITION {
                        selection_for_keys.set_selected(0);
                    }
                    gtk::glib::Propagation::Stop
                }
                gtk::gdk::Key::Up => {
                    let current_pos = selection_for_keys.selected();
                    if current_pos > 0 {
                        selection_for_keys.set_selected(current_pos - 1);
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

        // Connect list activation signal
        let launcher_for_activate = launcher.clone();
        let app_for_activate = app.clone();
        let list_store_for_activate = list_store.clone();
        list_view.connect_activate(move |_, position| {
            if let Some(obj) = list_store_for_activate.item(position) {
                if let Some(item_obj) = obj.downcast_ref::<LauncherItemObject>() {
                    let id = item_obj.id();
                    match launcher_for_activate.borrow().execute_item_by_id(&id) {
                        Ok(_) => app_for_activate.quit(),
                        Err(e) => eprintln!("Failed to launch app: {:?}", e),
                    }
                }
            }
        });

        // Initialize with default results
        let mut launcher_ref = launcher.borrow_mut();
        let results = launcher_ref.get_default_results();
        for entry in results.iter() {
            let item_obj = LauncherItemObject::new(
                entry.title(),
                entry.description(),
                entry.icon(),
                entry.id(),
            );
            list_store.append(&item_obj);
        }

        // Select the first item if available
        if list_store.n_items() > 0 {
            selection.set_selected(0);
        }
        drop(launcher_ref); // Release the borrow

        Self { window }
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
    println!("Icon: {}", icon_name);
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
                    .join(if (size != "scalable") {
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
                    .join(if (size != "scalable") {
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
