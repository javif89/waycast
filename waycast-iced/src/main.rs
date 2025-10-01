use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use iced::keyboard::key;
use iced::widget::{Column, button, column, image, row, scrollable, svg, text, text_input};
use iced::{Alignment, Element, Length, Size, Subscription, Task, event, keyboard, window};
use iced::widget::text_input::Id as TextInputId;
use waycast_core::WaycastLauncher;
use waycast_core::cache::CacheTTL;

static ICON_CACHE: OnceLock<Mutex<HashMap<String, IconHandle>>> = OnceLock::new();

fn get_or_load_icon(icon_name: &str) -> IconHandle {
    let cache = ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let cache_key = format!("icon:{}", icon_name);

    // Check if already cached
    if let Ok(cache_guard) = cache.lock() {
        if let Some(handle) = cache_guard.get(&cache_key) {
            return handle.clone();
        }
    }

    // Load the icon
    let icon_path = if let Some(p) = find_icon_file(icon_name, "48") {
        p
    } else {
        find_icon_file("application-x-executable", "48").unwrap_or_else(|| "notfound.png".into())
    };

    let handle = match Path::new(&icon_path).extension().and_then(|e| e.to_str()) {
        Some("svg") => IconHandle::Svg(svg::Handle::from_path(&icon_path)),
        _ => IconHandle::Image(image::Handle::from_path(&icon_path)),
    };

    // Store in cache
    if let Ok(mut cache_guard) = cache.lock() {
        cache_guard.insert(cache_key, handle.clone());
    }

    handle
}

pub fn main() -> iced::Result {
    iced::application("Waycast", Waycast::update, Waycast::view)
        .subscription(Waycast::subscription)
        .window(iced::window::Settings {
            size: Size {
                width: 800.,
                height: 500.,
            },
            position: iced::window::Position::Centered,
            decorations: false,
            resizable: false,
            transparent: false,
            level: iced::window::Level::AlwaysOnTop,
            ..iced::window::Settings::default()
        })
        .run_with(Waycast::init)
}

#[derive(Debug, Clone)]
enum Message {
    Search(String),
    DefaultList,
    Execute(String),
    KeyPressed(keyboard::Key),
    EventOccurred(iced::Event),
    CloseWindow,
    WindowFocused,
    SearchSubmit,
}

struct Waycast {
    launcher: WaycastLauncher,
    query: String,
    selected_index: usize,
    search_input_id: TextInputId,
}

#[derive(Clone)]
enum IconHandle {
    Svg(svg::Handle),
    Image(image::Handle),
}

impl Default for Waycast {
    fn default() -> Self {
        let mut projects = waycast_plugins::projects::new();
        let _ = projects.add_search_path("/home/javi/projects");
        let mut launcher = WaycastLauncher::new()
            .add_plugin(Box::new(waycast_plugins::drun::new()))
            .add_plugin(Box::new(waycast_plugins::file_search::new()))
            .add_plugin(Box::new(projects))
            .init();
        launcher.get_default_results();
        let query = String::new();

        // Initialize the icon cache
        ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

        Self {
            launcher,
            query,
            selected_index: 0,
            search_input_id: TextInputId::unique(),
        }
    }
}

impl Waycast {
    fn init() -> (Self, Task<Message>) {
        let app = Self::default();
        let focus_task = text_input::focus(app.search_input_id.clone());
        (app, focus_task)
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::DefaultList => {
                self.launcher.get_default_results();
                self.selected_index = 0;
                iced::Task::none()
            }
            Message::Search(query) => {
                self.query = query.clone();
                self.launcher.search(&query);
                self.selected_index = 0;
                iced::Task::none()
            }
            Message::Execute(id) => {
                match self.launcher.execute_item_by_id(&id) {
                    Ok(_) => println!("Executing app"),
                    Err(e) => println!("Error: {:#?}", e),
                }
                iced::exit()
            }
            Message::EventOccurred(event) => {
                if let iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key,
                    modifiers: _,
                    ..
                }) = event
                {
                    self.handle_key_press(key)
                } else {
                    iced::Task::none()
                }
            }
            Message::KeyPressed(_) => iced::Task::none(),
            Message::CloseWindow => iced::exit(),
            Message::WindowFocused => text_input::focus(self.search_input_id.clone()),
            Message::SearchSubmit => {
                // Execute the currently selected item
                if let Some(item) = self.launcher.current_results().get(self.selected_index) {
                    match self.launcher.execute_item_by_id(&item.id()) {
                        Ok(_) => println!("Executing app"),
                        Err(e) => println!("Error: {:#?}", e),
                    }
                    iced::exit()
                } else {
                    iced::Task::none()
                }
            }
        }
    }

    fn handle_key_press(&mut self, key: keyboard::Key) -> iced::Task<Message> {
        let results_len = self.launcher.current_results().len();

        match key {
            keyboard::Key::Named(key::Named::Escape) => {
                return iced::Task::done(Message::CloseWindow);
            }
            keyboard::Key::Named(key::Named::ArrowDown) => {
                if results_len > 0 {
                    self.selected_index = (self.selected_index + 1) % results_len;
                }
            }
            keyboard::Key::Named(key::Named::ArrowUp) => {
                if results_len > 0 {
                    if self.selected_index == 0 {
                        self.selected_index = results_len - 1;
                    } else {
                        self.selected_index -= 1;
                    }
                }
            }
            keyboard::Key::Named(key::Named::Enter) => {
                if let Some(item) = self.launcher.current_results().get(self.selected_index) {
                    match self.launcher.execute_item_by_id(&item.id()) {
                        Ok(_) => println!("Executing app"),
                        Err(e) => println!("Error: {:#?}", e),
                    }
                }
            }
            _ => {}
        }

        iced::Task::none()
    }

    fn view(&self) -> Element<Message> {
        let results = self.launcher.current_results();
        let icon_size = 32;

        let list = if results.is_empty() {
            Column::new().push(text("No results"))
        } else {
            let mut col = Column::new();
            for (index, i) in results.iter().enumerate() {
                let icon_handle = get_or_load_icon(&i.icon());

                let icon_view: Element<_> = match icon_handle {
                    IconHandle::Svg(handle) => svg::Svg::new(handle)
                        .width(icon_size)
                        .height(icon_size)
                        .into(),
                    IconHandle::Image(handle) => image::Image::new(handle)
                        .width(icon_size)
                        .height(icon_size)
                        .into(),
                };

                let row_ui = row![
                    column![icon_view].padding(5),
                    column![
                        text(i.title()).size(18),
                        text(i.description().unwrap_or_default()).size(14)
                    ]
                    .padding(5),
                ]
                .align_y(Alignment::Center);

                let is_selected = index == self.selected_index;

                let butt = button(row_ui)
                    .on_press(Message::Execute(i.id()))
                    .width(Length::Fill)
                    .style(if is_selected {
                        button::primary
                    } else {
                        button::secondary
                    });

                col = col.push(butt);
            }
            col
        };

        let scrollable_list = scrollable(list)
            .height(Length::Fill) // or Length::Fill if inside a fixed-height container
            .width(Length::Fill);

        column![
            text_input("Search...", &self.query)
                .id(self.search_input_id.clone())
                .on_input(Message::Search)
                .on_submit(Message::SearchSubmit),
            scrollable_list
        ]
        .into()
    }
}

fn find_icon_file(icon_name: &str, size: &str) -> Option<std::path::PathBuf> {
    // If icon_name is already a path and exists, return it directly
    let path = std::path::Path::new(icon_name);
    if path.exists() {
        return Some(path.to_path_buf());
    }

    let cache_key = format!("icon:{}:{}", icon_name, size);
    let cache = waycast_core::cache::get();

    let result = cache.remember_with_ttl(&cache_key, CacheTTL::hours(24), || {
        freedesktop::get_icon(icon_name)
    });

    if let Ok(opt_path) = result {
        return opt_path;
    }

    freedesktop::get_icon(icon_name)
}
