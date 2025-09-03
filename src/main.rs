use gio::prelude::*;
use gtk::gdk::Texture;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Entry, IconTheme, Image, Label, ListBox,
    Orientation, ScrolledWindow,
};
use gtk4_layer_shell as layerShell;
use layerShell::LayerShell;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use waycast::{LauncherListItem, drun};

struct AppModel {
    window: ApplicationWindow,
    list_box: ListBox,
    entries: Vec<Box<dyn LauncherListItem>>,
}

struct ListItem {
    text: String,
    icon: String,
}

impl ListItem {
    fn new(text: String, icon: String) -> Self {
        Self { text, icon }
    }

    fn create_widget(&self) -> GtkBox {
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
        // image.set_icon_name(Some("application-x-executable")); // Safe fallback

        let label = Label::new(Some(&self.text));
        label.set_xalign(0.0);

        container.append(&image);
        container.append(&label);
        container
    }
}

impl AppModel {
    fn new(app: &Application) -> Rc<RefCell<Self>> {
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

        let entries = drun::all();

        let model = Rc::new(RefCell::new(AppModel {
            window,
            list_box: list_box.clone(),
            entries,
        }));

        // Populate the list
        model.borrow().populate_list();

        // Connect search input signal
        let model_clone = model.clone();
        search_input.connect_changed(move |entry| {
            let query = entry.text().to_string();
            println!("query: {query}");
            model_clone.borrow().filter_list(&query);
        });

        model
    }

    fn populate_list(&self) {
        // Clear existing items
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        for entry in &self.entries {
            let list_item = ListItem::new(entry.title(), entry.icon());
            let widget = list_item.create_widget();
            self.list_box.append(&widget);
        }
    }

    fn filter_list(&self, query: &str) {
        // Clear existing items
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        let query_lower = query.to_lowercase();
        for entry in &self.entries {
            let title_lower = entry.title().to_lowercase();
            if query.is_empty() || title_lower.contains(&query_lower) {
                let list_item = ListItem::new(entry.title(), entry.icon());
                let widget = list_item.create_widget();
                self.list_box.append(&widget);
            }
        }
    }

    fn show(&self) {
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
    // let search_paths = [
    //     "/home/javi/.local/share/icons",
    //     "/home/javi/.icons",
    //     "/home/javi/.local/share/flatpak/exports/share/icons",
    //     "/var/lib/flatpak/exports/share/icons",
    //     "/home/javi/.nix-profile/share/icons",
    //     "/nix/profile/share/icons",
    //     "/home/javi/.local/state/nix/profile/share/icons",
    //     "/etc/profiles/per-user/javi/share/icons",
    //     "/nix/var/nix/profiles/default/share/icons",
    //     "/run/current-system/sw/share/icons",
    // ];

    // let pixmap_paths = [
    //     "/home/javi/.local/share/flatpak/exports/share/pixmaps",
    //     "/var/lib/flatpak/exports/share/pixmaps",
    //     "/home/javi/.nix-profile/share/pixmaps",
    //     "/nix/profile/share/pixmaps",
    //     "/home/javi/.local/state/nix/profile/share/pixmaps",
    //     "/etc/profiles/per-user/javi/share/pixmaps",
    //     "/nix/var/nix/profiles/default/share/pixmaps",
    //     "/run/current-system/sw/share/pixmaps",
    // ];

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

fn main() {
    let app = Application::builder()
        .application_id("dev.thegrind.waycast")
        .build();

    app.connect_activate(|app| {
        let model = AppModel::new(app);
        model.borrow().show();
    });

    app.run();

    // gtk::init().expect("Failed to init GTK");
    // let display = gtk::gdk::Display::default().unwrap();
    // let icon_theme = gtk::IconTheme::for_display(&display);
    // println!("Current icon theme: {:?}", icon_theme.theme_name());

    // for p in icon_theme.search_path() {
    //     println!("{}", p.to_string_lossy());
    // }
}
