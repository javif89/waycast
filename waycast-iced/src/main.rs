use std::path::Path;

use iced::widget::{
    Column, button, column, image, row, scrollable, svg,
    text, text_input,
};
use iced::{Element, Length, Size, Subscription, keyboard, event, Theme};
use iced::keyboard::key;
use waycast_core::cache::CacheTTL;
use waycast_core::WaycastLauncher;

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
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Search(String),
    DefaultList,
    Execute(String),
    KeyPressed(keyboard::Key),
    EventOccurred(iced::Event),
}

struct Waycast {
    launcher: WaycastLauncher,
    query: String,
    selected_index: usize,
}

impl Default for Waycast {
    fn default() -> Self {
        let mut launcher = WaycastLauncher::new()
            .add_plugin(Box::new(waycast_plugins::drun::new()))
            .add_plugin(Box::new(waycast_plugins::file_search::new()))
            .add_plugin(Box::new(waycast_plugins::projects::new()))
            .init();
        launcher.get_default_results();
        let query = String::new();
        Self { 
            launcher, 
            query,
            selected_index: 0,
        }
    }
}

impl Waycast {
    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::DefaultList => {
                self.launcher.get_default_results();
                self.selected_index = 0;
            }
            Message::Search(query) => {
                self.query = query.clone();
                self.launcher.search(&query);
                self.selected_index = 0;
            }
            Message::Execute(id) => match self.launcher.execute_item_by_id(&id) {
                Ok(_) => println!("Executing app"),
                Err(e) => println!("Error: {:#?}", e),
            },
            Message::EventOccurred(event) => {
                if let iced::Event::Keyboard(keyboard::Event::KeyPressed { 
                    key, 
                    modifiers: _,
                    .. 
                }) = event {
                    self.handle_key_press(key);
                }
            }
            Message::KeyPressed(_) => {}
        };
    }

    fn handle_key_press(&mut self, key: keyboard::Key) {
        let results_len = self.launcher.current_results().len();
        if results_len == 0 {
            return;
        }

        match key {
            keyboard::Key::Named(key::Named::ArrowDown) => {
                self.selected_index = (self.selected_index + 1) % results_len;
            }
            keyboard::Key::Named(key::Named::ArrowUp) => {
                if self.selected_index == 0 {
                    self.selected_index = results_len - 1;
                } else {
                    self.selected_index -= 1;
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
    }

    fn view(&self) -> Element<Message> {
        let results = self.launcher.current_results();

        let list = if results.is_empty() {
            Column::new().push(text("No results"))
        } else {
            results.iter().enumerate().fold(Column::new(), |col, (index, i)| {
                let icon_path = if let Some(p) = find_icon_file(&i.icon(), "48") {
                    p
                } else {
                    find_icon_file("application-x-executable", "48")
                        .unwrap_or_else(|| "notfound.png".into())
                };

                // Use SVG or raster image based on extension
                let icon_view: Element<_> =
                    match Path::new(&icon_path).extension().and_then(|e| e.to_str()) {
                        Some("svg") => svg::Svg::new(svg::Handle::from_path(&icon_path))
                            .width(48)
                            .height(48)
                            .into(),
                        _ => image::Image::new(image::Handle::from_path(&icon_path))
                            .width(48)
                            .height(48)
                            .into(),
                    };

                let row_ui = row![
                    column![icon_view].padding(10),
                    column![
                        text(i.title()).size(20),
                        text(i.description().unwrap_or_default()).size(14)
                    ]
                    .padding(10),
                ];

                let is_selected = index == self.selected_index;
                
                let butt = if is_selected {
                    button(row_ui)
                        .on_press(Message::Execute(i.id()))
                        .width(Length::Fill)
                        .style(button::primary)
                } else {
                    button(row_ui)
                        .on_press(Message::Execute(i.id()))
                        .width(Length::Fill)
                        .style(button::text)
                };
                
                col.push(butt)
            })
        };

        let scrollable_list = scrollable(list)
            .height(Length::Fill) // or Length::Fill if inside a fixed-height container
            .width(Length::Fill);

        column![
            text_input("Search...", &self.query).on_input(Message::Search),
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
