use crate::LauncherListItem;
use super::traits::{LauncherUI, UIEvent};
use gtk::gdk::Texture;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Entry, EventControllerKey, IconTheme, Image,
    Label, ListView, Orientation, ScrolledWindow, SignalListItemFactory, SingleSelection,
};
use gio::ListStore;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk4_layer_shell as layerShell;
use layerShell::LayerShell;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

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
        pub index: RefCell<usize>, // Store index to access original entry
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
    pub fn new(title: String, description: Option<String>, icon: String, index: usize) -> Self {
        let obj: Self = glib::Object::new();
        let imp = obj.imp();
        
        // Store the data
        *imp.title.borrow_mut() = title;
        *imp.description.borrow_mut() = description;
        *imp.icon.borrow_mut() = icon;
        *imp.index.borrow_mut() = index;
        
        obj
    }
    
    pub fn title(&self) -> String {
        self.imp().title.borrow().clone()
    }
    
    pub fn icon(&self) -> String {
        self.imp().icon.borrow().clone()
    }
    
    pub fn index(&self) -> usize {
        *self.imp().index.borrow()
    }
}

pub struct GtkLauncherUI {
    window: ApplicationWindow,
    list_view: ListView,
    list_store: ListStore,
    selection: SingleSelection,
    search_input: Entry,
    event_sender: Option<Sender<UIEvent>>,
    current_results: Vec<Box<dyn LauncherListItem>>, // Keep reference for indexing
}

impl GtkLauncherUI {
    pub fn new(app: &Application) -> Self {
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

        // Create the list store and selection model
        let list_store = ListStore::new::<LauncherItemObject>();
        let selection = SingleSelection::new(Some(list_store.clone()));

        // Create factory for rendering list items
        let factory = SignalListItemFactory::new();
        
        // Setup factory to create widgets
        factory.connect_setup(move |_, list_item| {
            let container = GtkBox::new(Orientation::Horizontal, 10);
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
                    if let Some(default) = find_icon_file("vscode", "48", &icon_theme) {
                        image = gtk::Image::from_file(default);
                    } else {
                        image = Image::from_icon_name("application-x-executable");
                    }
                }
                image.set_pixel_size(icon_size);
                
                // Create label
                let label = Label::new(Some(&item_obj.title()));
                label.set_xalign(0.0);
                
                child.append(&image);
                child.append(&label);
            }
        });
        
        let list_view = ListView::new(Some(selection.clone()), Some(factory));
        list_view.set_vexpand(true);
        list_view.set_can_focus(true);

        scrolled_window.set_child(Some(&list_view));
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

        // Set initial focus to search input so user can start typing immediately
        search_input.grab_focus();

        Self {
            window,
            list_view,
            list_store,
            selection,
            search_input,
            event_sender: None,
            current_results: Vec::new(),
        }
    }

    pub fn set_event_sender(&mut self, sender: Sender<UIEvent>) {
        self.event_sender = Some(sender.clone());

        // Connect search input signal
        let sender_clone = sender.clone();
        self.search_input.connect_changed(move |entry| {
            let query = entry.text().to_string();
            let _ = sender_clone.send(UIEvent::SearchChanged(query));
        });

        // Connect Enter key activation for search input
        let sender_clone = sender.clone();
        let selection_clone = self.selection.clone();
        self.search_input.connect_activate(move |_| {
            if let Some(selected_item) = selection_clone.selected_item() {
                if let Some(item_obj) = selected_item.downcast_ref::<LauncherItemObject>() {
                    let index = item_obj.index();
                    let _ = sender_clone.send(UIEvent::ItemActivated(index));
                }
            }
        });

        // Add key handler for launcher-style navigation
        let search_key_controller = EventControllerKey::new();
        let selection_clone = self.selection.clone();
        search_key_controller.connect_key_pressed(move |_controller, keyval, _keycode, _state| {
            match keyval {
                gtk::gdk::Key::Down => {
                    let current_pos = selection_clone.selected();
                    let n_items = selection_clone.model().unwrap().n_items();
                    if current_pos < n_items - 1 {
                        selection_clone.set_selected(current_pos + 1);
                    } else if n_items > 0 && current_pos == gtk::INVALID_LIST_POSITION {
                        selection_clone.set_selected(0);
                    }
                    gtk::glib::Propagation::Stop
                }
                gtk::gdk::Key::Up => {
                    let current_pos = selection_clone.selected();
                    if current_pos > 0 {
                        selection_clone.set_selected(current_pos - 1);
                    }
                    gtk::glib::Propagation::Stop
                }
                _ => gtk::glib::Propagation::Proceed,
            }
        });
        self.search_input.add_controller(search_key_controller);

        // Add ESC key handler at window level
        let window_key_controller = EventControllerKey::new();
        let sender_clone = sender.clone();
        window_key_controller.connect_key_pressed(move |_controller, keyval, _keycode, _state| {
            if keyval == gtk::gdk::Key::Escape {
                let _ = sender_clone.send(UIEvent::CloseRequested);
                gtk::glib::Propagation::Stop
            } else {
                gtk::glib::Propagation::Proceed
            }
        });
        self.window.add_controller(window_key_controller);

        // Connect list activation signal
        let sender_clone = sender;
        self.list_view.connect_activate(move |_, position| {
            let _ = sender_clone.send(UIEvent::ItemActivated(position as usize));
        });
    }
}

impl LauncherUI for GtkLauncherUI {
    fn show(&self) {
        self.window.present();
    }

    fn hide(&self) {
        self.window.close();
    }

    fn set_results(&mut self, results: &[Box<dyn LauncherListItem>]) {
        // Clear the list store
        self.list_store.remove_all();
        
        // Store results for indexing
        self.current_results = results.iter()
            .map(|item| {
                // Create a simple wrapper that implements the trait
                // This is a workaround since we can't clone trait objects
                Box::new(SimpleListItem {
                    title: item.title(),
                    description: item.description(),
                    icon: item.icon(),
                    original_ptr: 0, // Not used for execution
                }) as Box<dyn LauncherListItem>
            })
            .collect();
        
        // Add all entries to the store
        for (index, entry) in results.iter().enumerate() {
            let item_obj = LauncherItemObject::new(
                entry.title(),
                entry.description(),
                entry.icon(),
                index
            );
            self.list_store.append(&item_obj);
        }

        // Select the first item if available
        if self.list_store.n_items() > 0 {
            self.selection.set_selected(0);
        }
    }

    fn is_visible(&self) -> bool {
        self.window.is_visible()
    }
}

// Helper struct to store launcher item data
struct SimpleListItem {
    title: String,
    description: Option<String>,
    icon: String,
    original_ptr: usize, // Not used for execution, just for storage
}

impl LauncherListItem for SimpleListItem {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn description(&self) -> Option<String> {
        self.description.clone()
    }

    fn icon(&self) -> String {
        self.icon.clone()
    }

    fn execute(&self) -> Result<(), crate::LaunchError> {
        // This should never be called - the controller handles execution
        // using the original items
        Err(crate::LaunchError::CouldNotLaunch("Cannot execute from UI wrapper".into()))
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