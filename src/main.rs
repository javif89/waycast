use gio::prelude::*;
use gio::{File, FileIcon, Icon as GioIcon, ThemedIcon};
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Entry, Image, Label, ListBox, Orientation,
    ScrolledWindow,
};
use gtk::{IconLookupFlags, prelude::*};
use gtk4_layer_shell as layerShell;
use layerShell::LayerShell;
use std::cell::RefCell;
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

        // let icon = if self.icon.eq("com.discordapp.Discord")
        //     || self.icon.eq("preferences-desktop-theme")
        //     || self.icon.eq("solaar")
        //     || self.icon.eq("kvantum")
        // {
        //     println!("Failed: {}", self.icon);
        //     gio::Icon::for_string("vscode")
        // } else {
        //     let x = String::from(self.icon.clone());
        //     gio::Icon::for_string(&x)
        // };
        // // let icon = gio::Icon::for_string("kvantum");
        // let image: Image = match icon {
        //     Ok(ic) => Image::from_gicon(&ic),
        //     Err(_) => Image::from_icon_name("vscode"),
        // };
        // let image = Image::from_icon_name("vscode");
        // Use IconTheme for safe validation like Wofi does
        let display = gtk::gdk::Display::default().unwrap();
        let icon_theme = gtk::IconTheme::for_display(&display);

        // Get current paths and filter out problematic ones
        let current_paths = icon_theme.search_path();
        let clean_pathbufs: Vec<_> = current_paths
            .into_iter()
            .filter(|path| {
                let path_str = path.to_string_lossy();
                // Keep user paths, flatpak, and system paths - remove nix store
                !path_str.starts_with("/nix/store/")
                // !path_str.contains("patchelf") &&
                // !path_str.contains("vscode-1.") &&  // Individual app store paths
                // !path_str.contains("gsettings-desktop-schemas")
            })
            .collect();
        // Convert to &Path references for the API
        let clean_paths: Vec<&std::path::Path> =
            clean_pathbufs.iter().map(|p| p.as_path()).collect();

        println!("Filtered paths:");
        for path in &clean_paths {
            println!("  {}", path.to_string_lossy());
        }

        // Set the clean path list
        icon_theme.set_search_path(&clean_paths);

        if let Ok(gicon) = GioIcon::for_string(&self.icon) {
            // Check if it's a file icon (absolute path)
            if let Some(file_icon) = gicon.downcast_ref::<gio::FileIcon>() {
                let file = file_icon.file();
                if let Some(path) = file.path() {
                    if path.exists() {
                        println!("Loading file icon: {:?}", path);
                        // return gtk::Image::from_file(path);
                    }
                }
            }

            // Check if it's a themed icon
            if let Some(themed_icon) = gicon.downcast_ref::<gio::ThemedIcon>() {
                let icon_names = themed_icon.names();
                for name in &icon_names {
                    if icon_theme.has_icon(name) {
                        println!("Found themed icon: {}", name);
                        // let paintable = icon_theme.lookup_icon(
                        //     name,
                        //     &[], // Empty fallback array - let GTK handle it
                        //     icon_size,
                        //     scale,
                        //     gtk::TextDirection::None,
                        //     IconLookupFlags::empty(),
                        // );
                        // return Image::from_paintable(Some(&paintable));
                    }
                }
            }
        }

        let scale = container.scale_factor();
        let icon_size = 48;
        let lookup_flags = IconLookupFlags::empty();
        let image = if icon_theme.has_icon(&self.icon) {
            println!("Has icon: {}", self.icon);
            let paintable = icon_theme.lookup_icon(
                &self.icon,
                &["vscode"],
                icon_size,
                scale,
                gtk::TextDirection::None,
                lookup_flags,
            );
            if let Some(icon_name) = paintable.icon_name() {
                println!("Got icon name: {}", icon_name.to_string_lossy());
            }
            gtk::Image::from_paintable(Some(&paintable))
        } else {
            println!("No icon: {}", self.icon);
            gtk::Image::from_icon_name("application-x-executable")
        };
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

        println!("Starting to load desktop entries...");
        let entries = drun::all();
        println!("Found {} entries", entries.len());

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

        println!("Finished loading entries");
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

        // let display = gtk::gdk::Display::default().unwrap();
        // let icon_theme = gtk::IconTheme::for_display(&display);
        // Get current paths and filter out problematic ones
        // let current_paths = icon_theme.search_path();
        // let clean_pathbufs: Vec<_> = current_paths
        //     .into_iter()
        //     .filter(|path| {
        //         let path_str = path.to_string_lossy();
        //         // Keep user paths, flatpak, and system paths - remove nix store
        //         !path_str.starts_with("/nix/store/")
        //         // !path_str.contains("patchelf") &&
        //         // !path_str.contains("vscode-1.") &&  // Individual app store paths
        //         // !path_str.contains("gsettings-desktop-schemas")
        //     })
        //     .collect();
        // // Convert to &Path references for the API
        // let clean_paths: Vec<&std::path::Path> =
        //     clean_pathbufs.iter().map(|p| p.as_path()).collect();

        // println!("Filtered paths:");
        // for path in &clean_paths {
        //     println!("  {}", path.to_string_lossy());
        // }

        // // Set the clean path list
        // icon_theme.set_search_path(&clean_paths);

        // for p in icon_theme.search_path() {
        //     println!("{}", p.to_string_lossy());
        // }
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

// fn find_icon_file_directly(icon_name: &str) -> Option<std::path::PathBuf> {
//     // Search in the filtered paths manually, bypassing GTK entirely
//     let search_paths = [
//         "/home/javi/.local/share/icons",
//         "/home/javi/.icons",
//         "/home/javi/.local/share/flatpak/exports/share/icons",
//         "/var/lib/flatpak/exports/share/icons",
//         "/home/javi/.nix-profile/share/icons",
//         "/nix/profile/share/icons",
//         "/home/javi/.local/state/nix/profile/share/icons",
//         "/etc/profiles/per-user/javi/share/icons",
//         "/nix/var/nix/profiles/default/share/icons",
//         "/run/current-system/sw/share/icons",
//         "/home/javi/.local/share/flatpak/exports/share/pixmaps",
//         "/var/lib/flatpak/exports/share/pixmaps",
//         "/home/javi/.nix-profile/share/pixmaps",
//         "/nix/profile/share/pixmaps",
//         "/home/javi/.local/state/nix/profile/share/pixmaps",
//         "/etc/profiles/per-user/javi/share/pixmaps",
//         "/nix/var/nix/profiles/default/share/pixmaps",
//         "/run/current-system/sw/share/pixmaps",
//     ];

//     let sizes = ["48", "64", "32", "scalable"];
//     let categories = ["apps", "applications"];
//     let extensions = ["png", "svg", "xpm"];

//     for base_path in &search_paths {
//         let base = std::path::Path::new(base_path);
//         if !base.exists() {
//             continue;
//         }

//         // First try: direct pixmaps
//         for ext in &extensions {
//             let direct = base.join("pixmaps").join(format!("{}.{}", icon_name, ext));
//             if direct.exists() {
//                 println!("Found direct pixmap: {:?}", direct);
//                 return Some(direct);
//             }
//         }

//         // Second try: theme structure
//         if let Ok(theme_dirs) = std::fs::read_dir(base) {
//             for theme_entry in theme_dirs.flatten() {
//                 println!(
//                     "theme: {} in {}",
//                     theme_entry.file_name().to_string_lossy(),
//                     base.to_string_lossy()
//                 );
//                 let is_dir = theme_entry.metadata().map(|m| m.is_dir()).unwrap_or(false);
//                 if !is_dir {
//                     println!(
//                         "Skipping cuz not dir: {}",
//                         theme_entry.file_name().to_string_lossy()
//                     );
//                     continue;
//                 }

//                 let theme_path = theme_entry.path();

//                 for size in &sizes {
//                     for category in &categories {
//                         for ext in &extensions {
//                             let icon_path = theme_path
//                                 .join(size)
//                                 .join(category)
//                                 .join(format!("{}.{}", icon_name, ext));

//                             if icon_path.exists() {
//                                 println!("Found themed icon: {:?}", icon_path);
//                                 return Some(icon_path);
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }

//     None
// }
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

    let sizes = ["48", "64", "32", "scalable"];
    let categories = ["apps", "applications", "mimetypes"];
    let extensions = ["png", "svg", "xpm"];

    // First, search pixmaps directly (no subdirectories)
    for pixmap_path in &pixmap_paths {
        let base = std::path::Path::new(pixmap_path);
        if !base.exists() {
            continue;
        }

        for ext in &extensions {
            let direct_icon = base.join(format!("{}.{}", icon_name, ext));
            if direct_icon.exists() {
                println!("Found direct pixmap: {:?}", direct_icon);
                return Some(direct_icon);
            }
        }
    }

    // Then search icon theme directories
    for base_path in &search_paths {
        let base = std::path::Path::new(base_path);
        if !base.exists() {
            continue;
        }

        if let Ok(theme_dirs) = std::fs::read_dir(base) {
            for theme_entry in theme_dirs.flatten() {
                let theme_name = theme_entry.file_name().to_string_lossy().to_string();
                println!(
                    "Checking theme: {} in {}",
                    theme_name,
                    base.to_string_lossy()
                );

                let is_dir = theme_entry.path().is_dir();

                if !is_dir {
                    println!("Skipping no dir");
                    continue; // Skip files in icon directories
                }

                let theme_path = theme_entry.path();
                for size in &sizes {
                    for category in &categories {
                        println!(
                            "{}",
                            theme_path
                                .join(format!("{}x{}", size, size))
                                .join(category)
                                .to_string_lossy()
                        );
                        for ext in &extensions {
                            let icon_path = theme_path
                                .join(format!("{}x{}", size, size))
                                .join(category)
                                .join(format!("{}.{}", icon_name, ext));
                            if icon_path.exists() {
                                println!("Found themed icon: {:?}", icon_path);
                                return Some(icon_path);
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn main() {
    // let app = Application::builder()
    //     .application_id("dev.thegrind.waycast")
    //     .build();

    // app.connect_activate(|app| {
    //     let model = AppModel::new(app);
    //     model.borrow().show();
    // });

    // app.run();

    // for info in gio::AppInfo::all() {
    //     if !info.should_show() {
    //         continue;
    //     }

    //     println!("App: {}", info.display_name());
    //     if let Some(icon) = info.icon() {
    //         if let Some(x) = icon.to_string() {
    //             println!("Printed: {}", x.to_string());
    //         }
    //         if let Ok(ti) = icon.clone().downcast::<gio::ThemedIcon>() {
    //             // ThemedIcon may have multiple names, we take the first
    //             if let Some(name) = ti.names().first() {
    //                 println!("Themed: {}", name.to_string());
    //             }
    //         }

    //         if let Ok(fi) = icon.clone().downcast::<gio::FileIcon>() {
    //             if let Some(path) = fi.file().path() {
    //                 println!("File: {}", path.to_string_lossy().to_string());
    //             }
    //         }
    //     }
    //     println!("\n");
    // }

    // let scale = 1;
    // let icon_size = 48;
    // let lookup_flags = IconLookupFlags::empty();
    // let icon = gio::ThemedIcon::new("vscode");
    // let finded = icon_theme.lookup_by_gicon(
    //     &icon,
    //     icon_size,
    //     scale,
    //     gtk::TextDirection::None,
    //     lookup_flags,
    // );

    // let appinfo = gio::AppInfo::all();
    gtk::init().expect("Failed to init GTK");
    let display = gtk::gdk::Display::default().unwrap();
    let icon_theme = gtk::IconTheme::for_display(&display);
    // for i in icon_theme.icon_names() {
    //     println!("Icon: {}", i);
    //     for s in icon_theme.icon_sizes(i) {
    //         println!("- {}", s);
    //     }
    // }
    println!("Current icon theme: {:?}", icon_theme.theme_name());
    if let Some(path) = find_icon_file_directly("application-x-trash") {
        println!("Found at: {}", path.to_string_lossy());
    } else {
        println!("Not working");
    }
}
