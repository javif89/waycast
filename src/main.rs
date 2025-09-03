use gio::prelude::*;
use gio::{File, FileIcon, Icon as GioIcon, ThemedIcon};
use gtk::gdk::Texture;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Entry, Image, Label, ListBox, Orientation,
    ScrolledWindow,
};
use gtk::{IconLookupFlags, prelude::*};
use gtk4_layer_shell as layerShell;
use layerShell::LayerShell;
use std::cell::RefCell;
use std::fmt::format;
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

// TODO: I figured out what causes the stack overflow. Now to figure out
// why the icons for discord, solaar, and kvantum are not being found
impl ListItem {
    fn new(text: String, icon: String) -> Self {
        Self { text, icon }
    }

    fn create_widget(&self) -> GtkBox {
        let container = GtkBox::new(Orientation::Horizontal, 10);
        // let display = gtk::gdk::Display::default().unwrap();
        // let icon_theme = gtk::IconTheme::for_display(&display);

        // Get current paths and filter out problematic ones
        // TODO: Use this in the find_icon_file function
        // let current_paths = icon_theme.search_path();

        let icon_size = 48;
        let image: gtk::Image;
        if let Some(icon_path) = find_icon_file(&self.icon, "48", "Papirus") {
            println!("Found icon: {}", icon_path.to_string_lossy());
            // let file = gio::File::for_path(&icon_path);
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
            // image = match gtk::gdk::Texture::from_file(&file) {
            //     Ok(tex) => gtk::Image::from_paintable(Some(&tex)),
            //     Err(e) => {
            //         eprintln!("err: {}", e);
            //         Image::from_icon_name("application-x-executable")
            //     }
            // }
        } else {
            let default = find_icon_file("vscode", "48", "hicolor").unwrap();
            image = gtk::Image::from_file(default);
        }
        image.set_pixel_size(icon_size);
        // image.set_icon_name(Some("application-x-executable")); // Safe fallback

        let label = Label::new(Some(&self.text));
        label.set_xalign(0.0);

        container.append(&image);
        container.append(&label);
        println!("Icon: {}", self.icon);
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

fn find_icon_file(icon_name: &str, size: &str, theme_name: &str) -> Option<std::path::PathBuf> {
    let search_paths = [
        "/home/javi/.local/share/icons",
        "/home/javi/.icons",
        "/home/javi/.local/share/flatpak/exports/share/icons",
        "/var/lib/flatpak/exports/share/icons",
        "/home/javi/.nix-profile/share/icons",
        "/nix/profile/share/icons",
        "/home/javi/.local/state/nix/profile/share/icons",
        "/etc/profiles/per-user/javi/share/icons",
        "/nix/var/nix/profiles/default/share/icons",
        "/run/current-system/sw/share/icons",
    ];

    let pixmap_paths = [
        "/home/javi/.local/share/flatpak/exports/share/pixmaps",
        "/var/lib/flatpak/exports/share/pixmaps",
        "/home/javi/.nix-profile/share/pixmaps",
        "/nix/profile/share/pixmaps",
        "/home/javi/.local/state/nix/profile/share/pixmaps",
        "/etc/profiles/per-user/javi/share/pixmaps",
        "/nix/var/nix/profiles/default/share/pixmaps",
        "/run/current-system/sw/share/pixmaps",
    ];

    let sizes = [size, "scalable"];
    let categories = ["apps", "applications", "mimetypes"];
    let extensions = ["svg", "png", "xpm"];

    // First, search pixmaps directly (no subdirectories)
    for pixmap_path in &pixmap_paths {
        let base = std::path::Path::new(pixmap_path);
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

    // Build the search paths
    let mut search_in: Vec<PathBuf> = Vec::new();
    // Do all the theme directories first and high color second
    for path in &search_paths {
        let base = std::path::Path::new(path);
        for size in sizes {
            for cat in &categories {
                let path = base
                    .join(theme_name)
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
    for path in &search_paths {
        let base = std::path::Path::new(path);
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

    for s in &search_in {
        for ext in &extensions {
            let icon_path = s.join(format!("{}.{}", icon_name, ext));
            println!("- {}", format!("{}.{}", icon_name, ext));
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
    // for info in gio::AppInfo::all() {
    //     if !info.should_show() {
    //         continue;
    //     }

    //     println!("App: {}", info.display_name());
    //     if let Some(icon) = info.icon() {
    //         if let Some(x) = icon.to_string() {
    //             println!("Icon: {}", x.to_string());
    //             if let Some(path) = find_icon_file(&x, "48", icon_theme.theme_name().as_str()) {
    //                 println!("Found at: {}", path.to_string_lossy());
    //             } else {
    //                 println!("Not found");
    //             }
    //         }
    //         // if let Ok(ti) = icon.clone().downcast::<gio::ThemedIcon>() {
    //         //     // ThemedIcon may have multiple names, we take the first
    //         //     if let Some(name) = ti.names().first() {
    //         //         println!("Themed: {}", name.to_string());
    //         //     }
    //         // }

    //         // if let Ok(fi) = icon.clone().downcast::<gio::FileIcon>() {
    //         //     if let Some(path) = fi.file().path() {
    //         //         println!("File: {}", path.to_string_lossy().to_string());
    //         //     }
    //         // }
    //     }
    //     println!("\n");
    // }

    // let appinfo = gio::AppInfo::all();
}
