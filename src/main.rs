use gio::{Icon, prelude::*};
use gtk::{
    Box as GtkBox, CssProvider, Entry, IconLookupFlags, IconTheme, Image, Label, ListBox, ListView,
    NoSelection, Orientation, STYLE_PROVIDER_PRIORITY_APPLICATION, ScrolledWindow,
    SignalListItemFactory, StringList, StringObject, Window,
};
use gtk::{Button, prelude::*};
use gtk4_layer_shell as layerShell;
use layerShell::LayerShell;
use relm4::factory::{FactoryComponent, FactorySender, FactoryVecDeque};
use relm4::{Component, ComponentParts, RelmApp, RelmWidgetExt, SimpleComponent, component};
use waycast::{LaunchError, LauncherListItem, drun};

struct AppModel {
    list_items: FactoryVecDeque<ListItem>,
}

#[derive(Debug)]
enum AppMsg {
    TextEntered(String),
    ListItemSelected(String),
    None,
}

#[derive(Debug)]
struct ListItem {
    text: String,
    icon: String,
}

#[relm4::factory]
impl FactoryComponent for ListItem {
    type Init = (String, String);
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[root]
        GtkBox {
            set_orientation: Orientation::Horizontal,
            set_spacing: 10,

            Image {
                set_pixel_size: 50,
                set_icon_name: Some(self.icon.as_str()),
            },

            Label {
                set_xalign: 0.0,
                set_label: &self.text
            },
        }
    }

    fn init_model(
        (text, icon): Self::Init,
        _index: &Self::Index,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self { text, icon }
    }
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = StringList;
    type Input = AppMsg;
    type Output = ();

    view! {
        #[name = "launcher_window"]
        Window {
            set_title: Some("Waycast"),
            set_default_width: 800,
            set_default_height: 500,
            set_resizable: false,

            GtkBox {
                set_orientation: Orientation::Vertical,

                #[name = "search_input"]
                Entry {
                    set_placeholder_text: Some("Search..."),
                    connect_changed[sender] => move |e| {
                        sender.input(AppMsg::TextEntered(e.text().to_string()));
                    }
                },

                ScrolledWindow {
                    set_min_content_height: 300,

                    #[local_ref]
                    items -> ListBox {
                        set_vexpand: true,
                    }
                }
            }
        }
    }

    fn init(
        _list_items: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let mut list_items: FactoryVecDeque<ListItem> = FactoryVecDeque::builder()
            .launch(ListBox::default())
            .forward(sender.input_sender(), |_| AppMsg::None);
        {
            let mut guard = list_items.guard();
            println!("Starting to load desktop entries...");
            let entries = drun::all();
            println!("Found {} entries", entries.len());
            for p in entries {
                guard.push_back((p.title(), p.icon()));
            }
            println!("Finished loading entries");
        }
        let model = AppModel { list_items };
        let items = model.list_items.widget();
        let widgets = view_output!();
        // Set up layer shell so the launcher can float
        // like it's supposed to.
        widgets.launcher_window.init_layer_shell();
        let edges = [
            layerShell::Edge::Top,
            layerShell::Edge::Bottom,
            layerShell::Edge::Left,
            layerShell::Edge::Right,
        ];
        for edge in edges {
            widgets.launcher_window.set_anchor(edge, false);
        }
        widgets
            .launcher_window
            .set_keyboard_mode(layerShell::KeyboardMode::OnDemand);
        widgets.launcher_window.set_layer(layerShell::Layer::Top);
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: relm4::ComponentSender<Self>) {
        match message {
            AppMsg::TextEntered(query) => {
                println!("query: {query}");
            }
            _ => unimplemented!(),
        }
    }
}

macro_rules! yesno {
    ($var:expr) => {
        if $var { "Yes" } else { "No" }
    };
}

fn main() {
    let app = RelmApp::new("dev.thegrind.waycast");
    app.run::<AppModel>(StringList::new(&[]));
    // let entries = drun::all();
    // for e in &entries {
    //     println!("---------------------");
    //     println!("App: {}", e.title());
    //     println!("Icon: {}", e.icon().unwrap_or("<NONE>".into()));
    // }
}
