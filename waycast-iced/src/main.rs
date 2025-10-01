use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use iced::border::Radius;
use iced::keyboard::key;
use iced::widget::scrollable::{Id as ScrollableId, Rail};
use iced::widget::text_input::Id as TextInputId;
use iced::widget::{
    Column, Row, Rule, button, center, column, container, image, row, rule, scrollable, svg, text,
    text_input, vertical_space,
};
use iced::{
    Alignment, Border, Color, Element, Length, Subscription, Task as Command, Theme, event,
    keyboard,
};
use iced_layershell::Application;
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;
use waycast_core::WaycastLauncher;
use waycast_core::cache::CacheTTL;

static ICON_CACHE: OnceLock<Mutex<HashMap<String, IconHandle>>> = OnceLock::new();

pub fn main() -> Result<(), iced_layershell::Error> {
    Waycast::run(Settings {
        id: Some("Waycast".into()),
        layer_settings: LayerShellSettings {
            size: Some((800, 500)),
            exclusive_zone: 0,
            anchor: Anchor::Bottom | Anchor::Left | Anchor::Right | Anchor::Top,
            start_mode: StartMode::Active,
            ..Default::default()
        },
        ..Default::default()
    })
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    Search(String),
    Execute(String),
    EventOccurred(iced::Event),
    CloseWindow,
    SearchSubmit,
}

struct Waycast {
    launcher: WaycastLauncher,
    query: String,
    selected_index: usize,
    search_input_id: TextInputId,
    scrollable_id: ScrollableId,
}

#[derive(Clone)]
enum IconHandle {
    Svg(svg::Handle),
    Image(image::Handle),
}

impl Application for Waycast {
    type Message = Message;
    type Flags = ();
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Self, Command<Message>) {
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

        let search_input_id = TextInputId::unique();
        let scrollable_id = ScrollableId::unique();
        let app = Self {
            launcher,
            query,
            selected_index: 0,
            search_input_id: search_input_id.clone(),
            scrollable_id,
        };
        let focus_task = text_input::focus(search_input_id);
        (app, focus_task)
    }

    fn namespace(&self) -> String {
        String::from("Waycast")
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            event::listen().map(Message::EventOccurred),
            keyboard::on_key_release(|key, _modifiers| {
                if matches!(key, keyboard::Key::Named(key::Named::Escape)) {
                    Some(Message::CloseWindow)
                } else {
                    None
                }
            }),
        ])
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Search(query) => {
                self.query = query.clone();
                self.launcher.search(&query);
                self.selected_index = 0;
                Command::none()
            }
            Message::Execute(id) => {
                match self.launcher.execute_item_by_id(&id) {
                    Ok(_) => println!("Executing app"),
                    Err(e) => println!("Error: {:#?}", e),
                }
                std::process::exit(0)
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
                    Command::none()
                }
            }
            Message::CloseWindow => std::process::exit(0),
            Message::SearchSubmit => {
                // Execute the currently selected item
                if let Some(item) = self.launcher.current_results().get(self.selected_index) {
                    match self.launcher.execute_item_by_id(&item.id()) {
                        Ok(_) => println!("Executing app"),
                        Err(e) => println!("Error: {:#?}", e),
                    }
                    std::process::exit(0)
                } else {
                    Command::none()
                }
            }
            _ => unreachable!(),
        }
    }

    fn view(&self) -> Element<Message> {
        let results = self.launcher.current_results();
        let icon_size = 48;

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
                        button::text
                    });

                col = col.push(butt);
            }
            col
        };

        let rail = Rail {
            scroller: scrollable::Scroller {
                color: Color::TRANSPARENT,
                border: Border {
                    color: Color::TRANSPARENT,
                    ..Default::default()
                },
            },
            background: None,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.,
                ..Default::default()
            },
        };
        let scrollable_list = scrollable(list)
            .id(self.scrollable_id.clone())
            .height(Length::Fill)
            .width(Length::Fill)
            .style(move |_, _| scrollable::Style {
                container: container::Style::default(),
                gap: None,
                horizontal_rail: rail.to_owned(),
                vertical_rail: rail.to_owned(),
            });
        // let pinned = {
        //     let mut col = Row::new();
        //     for (index, i) in results.iter().enumerate().take(5) {
        //         let icon_handle = get_or_load_icon(&i.icon());

        //         let icon_view: Element<_> = match icon_handle {
        //             IconHandle::Svg(handle) => svg::Svg::new(handle)
        //                 .width(icon_size)
        //                 .height(icon_size)
        //                 .into(),
        //             IconHandle::Image(handle) => image::Image::new(handle)
        //                 .width(icon_size)
        //                 .height(icon_size)
        //                 .into(),
        //         };

        //         let row_ui = row![column![icon_view].padding(5).align_x(Alignment::Center),]
        //             .align_y(Alignment::Center);

        //         let butt = button(center(row_ui).width(Length::Shrink).height(Length::Shrink))
        //             .on_press(Message::Execute(i.id()))
        //             .width(Length::Fill)
        //             .style(button::text);

        //         col = col.push(butt);
        //     }
        //     col
        // };

        column![
            container(
                text_input("Search...", &self.query)
                    .id(self.search_input_id.clone())
                    .size(25)
                    .padding(5)
                    .style(|theme: &Theme, _| {
                        text_input::Style {
                            background: iced::Background::Color(theme.palette().background),
                            border: Border {
                                color: Color::TRANSPARENT,
                                width: 0.0,
                                radius: Radius {
                                    top_left: 0.,
                                    top_right: 0.,
                                    bottom_left: 0.,
                                    bottom_right: 0.,
                                },
                            },
                            icon: Color::WHITE,
                            placeholder: Color::from_rgba(255., 255., 255., 0.3),
                            selection: Color::BLACK,
                            value: Color::WHITE,
                        }
                    })
                    .on_input(Message::Search)
                    .on_submit(Message::SearchSubmit)
            )
            .padding(20),
            // pinned,
            // Rule::horizontal(5).style(|_| {
            //     rule::Style {
            //         // color: Color::from_rgba(255., 255., 255., 0.3),
            //         color: Color::WHITE,
            //         width: 1,
            //         radius: Radius::default(),
            //         fill_mode: rule::FillMode::Full,
            //     }
            // }),
            vertical_space().height(10),
            scrollable_list
        ]
        .into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    // fn style(&self, theme: &Self::Theme) -> iced_layershell::Appearance {
    //     use iced_layershell::Appearance;
    //     Appearance {
    //         background_color: Color::TRANSPARENT,
    //         text_color: theme.palette().text,
    //     }
    // }
}

impl Waycast {
    fn handle_key_press(&mut self, key: keyboard::Key) -> Command<Message> {
        let results_len = self.launcher.current_results().len();

        match key {
            keyboard::Key::Named(key::Named::ArrowDown) => {
                self.selected_index = (self.selected_index + 1).min(results_len);
                // Scroll to make the selected item visible
                let item_height = 60.0;
                let scroll_offset = self.selected_index as f32 * item_height;
                return scrollable::scroll_to(
                    self.scrollable_id.clone(),
                    scrollable::AbsoluteOffset {
                        x: 0.0,
                        y: scroll_offset,
                    },
                );
            }
            keyboard::Key::Named(key::Named::ArrowUp) => {
                self.selected_index = self.selected_index.saturating_sub(1);
                // Scroll to make the selected item visible
                let item_height = 60.0;
                let scroll_offset = self.selected_index as f32 * item_height;
                return scrollable::scroll_to(
                    self.scrollable_id.clone(),
                    scrollable::AbsoluteOffset {
                        x: 0.0,
                        y: scroll_offset,
                    },
                );
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

        Command::none()
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
