use iced::keyboard::key;
use iced::widget::scrollable::{self, Id as ScrollableId};
use iced::widget::text_input::{self, Id as TextInputId};
use iced::widget::{
    button, column, container, image, row, scrollable as scrollable_widget, svg, text,
    text_input as text_input_widget,
};
use iced::{Alignment, Element, Length, Subscription, Task as Command, Theme, event, keyboard};
use iced_layershell::Application;
use iced_layershell::to_layer_message;
use waycast_core::WaycastLauncher;

use crate::config;
use crate::icons::{self, IconHandle};
use crate::ui::styles;

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    Search(String),
    Execute(String),
    EventOccurred(iced::Event),
    CloseWindow,
    SearchSubmit,
}

pub struct Waycast {
    launcher: WaycastLauncher,
    query: String,
    selected_index: usize,
    search_input_id: TextInputId,
    scrollable_id: ScrollableId,
    should_hide: bool,
}

impl Application for Waycast {
    type Message = Message;
    type Flags = ();
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut launcher = init_launcher();
        let search_input_id = TextInputId::unique();
        let scrollable_id = ScrollableId::unique();

        launcher.get_default_results();

        let app = Self {
            launcher,
            query: String::new(),
            selected_index: 0,
            search_input_id: search_input_id.clone(),
            scrollable_id,
            should_hide: false,
        };

        let focus_task = text_input::focus(search_input_id);
        (app, focus_task)
    }

    fn namespace(&self) -> String {
        config::APP_NAME.to_string()
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
                if query.trim().is_empty() {
                    // Like GTK UI: show default results when query is empty
                    self.launcher.get_default_results();
                } else {
                    self.launcher.search(&query);
                }
                self.selected_index = 0;
                Command::none()
            }
            Message::Execute(_id) => self.execute_item(),
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
            Message::CloseWindow => iced::exit(),
            Message::SearchSubmit => self.execute_item(),
            _ => Command::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        if self.should_hide {
            // Return empty view to hide window content immediately
            return column![].into();
        }

        let results_list = self.build_results_list();
        let scrollable_list = self.build_scrollable(results_list);
        let search_input = self.build_search_input();

        let mut col = column![container(search_input).padding(config::PADDING_LARGE),];

        if let Ok(calc_result) = mathengine::evaluate_expression(&self.query) {
            col = col.push(
                text(format!("= {}", calc_result))
                    .size(20)
                    .font(styles::bold_font()),
            );
        }

        col = col.push(container(scrollable_list).padding(config::PADDING_LARGE));

        col.into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}

impl Waycast {
    fn handle_key_press(&mut self, key: keyboard::Key) -> Command<Message> {
        let results_len = self.launcher.current_results().len();
        if results_len == 0 {
            return Command::none();
        }

        match key {
            keyboard::Key::Named(key::Named::ArrowDown) => {
                self.selected_index = (self.selected_index + 1) % results_len;
                self.scroll_to_selected()
            }
            keyboard::Key::Named(key::Named::ArrowUp) => {
                if self.selected_index == 0 {
                    self.selected_index = results_len - 1;
                } else {
                    self.selected_index -= 1;
                }
                self.scroll_to_selected()
            }
            keyboard::Key::Named(key::Named::Enter) => self.execute_item(),
            _ => Command::none(),
        }
    }

    fn execute_item(&self) -> Command<Message> {
        println!("Executing");
        if let Some(item) = self.launcher.current_results().get(self.selected_index) {
            // TODO: Show some error in the UI if launching fails
            if let Err(e) = item.execute() {
                eprintln!("Error executing app {} | {:#?}", item.id(), e);
            }
        }

        iced::exit()
    }

    fn scroll_to_selected(&self) -> Command<Message> {
        let scroll_offset = self.selected_index as f32 * config::ITEM_HEIGHT;
        scrollable::scroll_to(
            self.scrollable_id.clone(),
            scrollable::AbsoluteOffset {
                x: 0.0,
                y: scroll_offset,
            },
        )
    }

    fn build_search_input(&self) -> Element<'_, Message> {
        row![
            text_input_widget(config::SEARCH_PLACEHOLDER, &self.query)
                .id(self.search_input_id.clone())
                .size(config::SEARCH_INPUT_SIZE)
                .padding(config::PADDING_SMALL)
                .style(styles::search_input_style)
                .on_input(Message::Search)
                .width(Length::Fill)
                .on_submit(Message::SearchSubmit),
        ]
        .into()
    }

    fn build_results_list(&self) -> Element<'_, Message> {
        let results = self.launcher.current_results();

        if results.is_empty() {
            return column![text("No results")].into();
        }

        let mut col = column![];
        for (index, item) in results.iter().enumerate() {
            let result_item = self.build_result_item(item.as_ref(), index == self.selected_index);
            col = col.push(result_item);
        }

        col.into()
    }

    fn build_result_item(
        &self,
        item: &dyn waycast_core::LauncherListItem,
        is_selected: bool,
    ) -> Element<'_, Message> {
        let icon_handle = icons::get_or_load_icon(&item.icon());
        let icon_view = build_icon_view(icon_handle);

        let content = row![
            column![icon_view].padding(config::PADDING_SMALL),
            column![
                text(item.title())
                    .size(config::TITLE_FONT_SIZE)
                    .font(styles::bold_font()),
                text(item.description().unwrap_or_default())
                    .size(config::DESCRIPTION_FONT_SIZE)
                    .font(styles::italic_font())
            ]
            .padding(config::PADDING_SMALL),
        ]
        .align_y(Alignment::Center);

        button(content)
            .on_press(Message::Execute(item.id()))
            .width(Length::Fill)
            .style(styles::result_button_style(is_selected))
            .into()
    }

    fn build_scrollable<'a>(&self, content: Element<'a, Message>) -> Element<'a, Message> {
        scrollable_widget(content)
            .id(self.scrollable_id.clone())
            .height(Length::Fill)
            .width(Length::Fill)
            .style(styles::scrollable_style)
            .into()
    }
}

fn init_launcher() -> WaycastLauncher {
    // Initialize launcher exactly like the GTK UI does

    WaycastLauncher::new()
        .add_plugin(Box::new(waycast_plugins::drun::new()))
        .add_plugin(Box::new(waycast_plugins::file_search::new()))
        .add_plugin(Box::new(waycast_plugins::projects::new()))
        .init()
}

fn build_icon_view(icon_handle: IconHandle) -> Element<'static, Message> {
    match icon_handle {
        IconHandle::Svg(handle) => svg::Svg::new(handle)
            .width(config::ICON_SIZE)
            .height(config::ICON_SIZE)
            .into(),
        IconHandle::Image(handle) => image::Image::new(handle)
            .width(config::ICON_SIZE)
            .height(config::ICON_SIZE)
            .into(),
    }
}
