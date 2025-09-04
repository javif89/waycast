use crate::{LauncherListItem, LauncherPlugin};
use gtk::gdk::Texture;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Entry, EventControllerKey, IconTheme, Image,
    Label, ListBox, Orientation, ScrolledWindow,
};
use gtk4_layer_shell as layerShell;
use layerShell::LayerShell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
mod launcher_builder;
use launcher_builder::WaycastLauncherBuilder;
use std::sync::Arc;

pub struct WaycastLauncher {
    pub window: ApplicationWindow,
    pub list_box: ListBox,
    pub entries: Vec<Box<dyn LauncherListItem>>,
    // All plugins
    pub plugins: Vec<Arc<dyn LauncherPlugin>>,
    // Plugins with by_prefix_only()->false
    pub plugins_show_always: Vec<Arc<dyn LauncherPlugin>>,
    // Prefix hash map
    pub plugins_by_prefix: HashMap<String, Arc<dyn LauncherPlugin>>,
}

impl WaycastLauncher {
    pub fn new() -> WaycastLauncherBuilder {
        WaycastLauncherBuilder {
            plugins: Vec::new(),
        }
    }
}

pub struct ListItem {
    text: String,
    icon: String,
}

impl ListItem {
    pub fn new(text: String, icon: String) -> Self {
        Self { text, icon }
    }

    pub fn create_widget(&self) -> GtkBox {
        let container = GtkBox::new(Orientation::Horizontal, 10);
        let display = gtk::gdk::Display::default().unwrap();
        let icon_theme = gtk::IconTheme::for_display(&display);

        let icon_size = 48;
        let image: gtk::Image;
        if let Some(icon_path) = find_icon_file(&self.icon, "48", &icon_theme) {
            image = match Pixbuf::from_file_at_scale(icon_path, icon_size, icon_size, true) {
                Ok(pb) => {
                    let tex = Texture::for_pixbuf(&pb);
                    gtk::Image::from_paintable(Some(&tex))
                }
                Err(e) => {
                    eprintln!("err: {}", e);
                    Image::from_icon_name("application-x-executable")
                }
            }
        } else {
            let default = find_icon_file("vscode", "48", &icon_theme).unwrap();
            image = gtk::Image::from_file(default);
        }
        image.set_pixel_size(icon_size);

        let label = Label::new(Some(&self.text));
        label.set_xalign(0.0);

        container.append(&image);
        container.append(&label);
        container
    }
}

impl WaycastLauncher {
    fn create_with_plugins(
        app: &Application,
        init_plugins: Vec<Box<dyn LauncherPlugin>>,
    ) -> Rc<RefCell<Self>> {
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

        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_min_content_height(300);

        let list_box = ListBox::new();
        list_box.set_vexpand(true);
        list_box.set_can_focus(true);
        list_box.set_activate_on_single_click(false);

        scrolled_window.set_child(Some(&list_box));
        main_box.append(&search_input);
        main_box.append(&scrolled_window);
        window.set_child(Some(&main_box));

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

        let mut plugins: Vec<Arc<dyn LauncherPlugin>> = Vec::new();
        for p in init_plugins {
            plugins.push(Arc::from(p));
        }
        // Organize plugins for faster querying
        let mut plugins_show_always: Vec<Arc<dyn LauncherPlugin>> = Vec::new();
        for p in &plugins {
            if !p.by_prefix_only() {
                plugins_show_always.push(Arc::clone(p));
            }
        }

        let mut plugins_by_prefix: HashMap<String, Arc<dyn LauncherPlugin>> = HashMap::new();
        for p in &plugins {
            if let Some(prefix) = p.prefix() {
                plugins_by_prefix.insert(prefix, Arc::clone(p));
            }
        }

        // Init the launcher model
        let entries: Vec<Box<dyn LauncherListItem>> = Vec::new();
        let model: Rc<RefCell<WaycastLauncher>> = Rc::new(RefCell::new(WaycastLauncher {
            window,
            list_box: list_box.clone(),
            entries,
            plugins,
            plugins_show_always,
            plugins_by_prefix,
        }));

        // Populate the list
        model.borrow_mut().populate_list();

        // Set initial focus to search input so user can start typing immediately
        search_input.grab_focus();

        // Connect search input signal
        let model_clone = model.clone();
        search_input.connect_changed(move |entry| {
            let query = entry.text().to_string();
            println!("query: {query}");
            model_clone.borrow_mut().filter_list(&query);
        });

        // Connect Enter key activation for search input
        let list_box_clone_for_activate = list_box.clone();
        let model_clone_for_activate = model.clone();
        search_input.connect_activate(move |_| {
            println!("Search entry activated!");
            if let Some(selected_row) = list_box_clone_for_activate.selected_row() {
                let index = selected_row.index() as usize;
                let model_ref = model_clone_for_activate.borrow();
                if let Some(entry) = model_ref.entries.get(index) {
                    println!("Launching app: {}", entry.title());
                    match entry.execute() {
                        Ok(_) => {
                            println!("App launched successfully, closing launcher");
                            model_ref.window.close();
                        }
                        Err(e) => {
                            eprintln!("Failed to launch app: {:?}", e);
                        }
                    }
                }
            }
        });

        // Add key handler for launcher-style navigation
        let search_key_controller = EventControllerKey::new();
        let list_box_clone_for_search = list_box.clone();
        search_key_controller.connect_key_pressed(move |_controller, keyval, _keycode, _state| {
            match keyval {
                gtk::gdk::Key::Down => {
                    // Move to next item in list
                    if let Some(selected_row) = list_box_clone_for_search.selected_row() {
                        let index = selected_row.index();
                        if let Some(next_row) = list_box_clone_for_search.row_at_index(index + 1) {
                            list_box_clone_for_search.select_row(Some(&next_row));
                        }
                    } else if let Some(first_row) = list_box_clone_for_search.row_at_index(0) {
                        list_box_clone_for_search.select_row(Some(&first_row));
                    }
                    gtk::glib::Propagation::Stop
                }
                gtk::gdk::Key::Up => {
                    // Move to previous item in list
                    if let Some(selected_row) = list_box_clone_for_search.selected_row() {
                        let index = selected_row.index();
                        if index > 0 {
                            if let Some(prev_row) =
                                list_box_clone_for_search.row_at_index(index - 1)
                            {
                                list_box_clone_for_search.select_row(Some(&prev_row));
                            }
                        }
                    }
                    gtk::glib::Propagation::Stop
                }
                _ => gtk::glib::Propagation::Proceed,
            }
        });
        search_input.add_controller(search_key_controller);

        // Add simple ESC key handler at window level
        let window_key_controller = EventControllerKey::new();
        let window_clone = model.borrow().window.clone();
        window_key_controller.connect_key_pressed(move |_controller, keyval, _keycode, _state| {
            if keyval == gtk::gdk::Key::Escape {
                window_clone.close();
                gtk::glib::Propagation::Stop
            } else {
                gtk::glib::Propagation::Proceed
            }
        });
        model.borrow().window.add_controller(window_key_controller);

        // Connect row activation signal to launch app and close launcher
        let model_clone_2 = model.clone();
        list_box.connect_row_activated(move |_, row| {
            let index = row.index() as usize;
            let model_ref = model_clone_2.borrow();
            if let Some(entry) = model_ref.entries.get(index) {
                println!("Launching app: {}", entry.title());
                match entry.execute() {
                    Ok(_) => {
                        println!("App launched successfully, closing launcher");
                        model_ref.window.close();
                    }
                    Err(e) => {
                        eprintln!("Failed to launch app: {:?}", e);
                    }
                }
            }
        });

        model
    }

    pub fn clear_list_ui(&self) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
    }

    pub fn render_list(&self) {
        self.clear_list_ui();
        for entry in &self.entries {
            let list_item = ListItem::new(entry.title(), entry.icon());
            let widget = list_item.create_widget();
            self.list_box.append(&widget);
        }

        // Always select the first item if available
        if let Some(first_row) = self.list_box.row_at_index(0) {
            self.list_box.select_row(Some(&first_row));
        }
    }

    pub fn populate_list(&mut self) {
        self.entries.clear();
        for plugin in &self.plugins_show_always {
            for entry in plugin.default_list() {
                self.entries.push(entry);
            }
        }

        self.render_list();
    }

    pub fn filter_list(&mut self, query: &str) {
        self.entries.clear();

        for plugin in &self.plugins {
            for entry in plugin.filter(query) {
                self.entries.push(entry);
            }
        }

        self.render_list();
    }

    pub fn show(&self) {
        self.window.present();
    }
}

fn find_icon_file(
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
                    .join(if !(size == "scalable".to_string()) {
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
                    .join(if !(size == "scalable".to_string()) {
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
