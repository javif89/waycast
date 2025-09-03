use gio::prelude::*;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Entry, Image, Label, ListBox, Orientation,
    ScrolledWindow,
};
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

impl ListItem {
    fn new(text: String, icon: String) -> Self {
        Self { text, icon }
    }

    fn create_widget(&self) -> GtkBox {
        let container = GtkBox::new(Orientation::Horizontal, 10);

        let image = Image::new();
        image.set_pixel_size(50);
        image.set_icon_name(Some(&self.icon));

        let label = Label::new(Some(&self.text));
        label.set_xalign(0.0);

        // container.append(&image);
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

fn main() {
    let app = Application::builder()
        .application_id("dev.thegrind.waycast")
        .build();

    app.connect_activate(|app| {
        let model = AppModel::new(app);
        model.borrow().show();
    });

    app.run();
}
