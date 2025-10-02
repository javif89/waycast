use iced::border::Radius;
use iced::widget::{button, container, scrollable, text_input};
use iced::{Background, Border, Color, Font, Theme};

use crate::config;

pub fn bold_font() -> Font {
    Font {
        weight: iced::font::Weight::Bold,
        ..Font::DEFAULT
    }
}

pub fn italic_font() -> Font {
    Font {
        style: iced::font::Style::Italic,
        ..Font::DEFAULT
    }
}

pub fn search_input_style(_theme: &Theme, _status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius::default(),
        },
        icon: Color::WHITE,
        placeholder: config::PLACEHOLDER_COLOR,
        selection: Color::BLACK,
        value: Color::WHITE,
    }
}

pub fn result_button_style(is_selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme: &Theme, status: button::Status| {
        let base = button::text(theme, status);

        if is_selected {
            button::Style {
                background: Some(Background::Color(config::SELECTED_BG_COLOR)),
                text_color: config::SELECTED_TEXT_COLOR,
                ..base
            }
        } else {
            base
        }
    }
}

pub fn transparent_scrollbar() -> scrollable::Rail {
    scrollable::Rail {
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
    }
}

pub fn scrollable_style(_theme: &Theme, _status: scrollable::Status) -> scrollable::Style {
    let rail = transparent_scrollbar();
    scrollable::Style {
        container: container::Style::default(),
        gap: None,
        horizontal_rail: rail.clone(),
        vertical_rail: rail,
    }
}
