use iced::Color;

pub struct WaycastTheme;

impl WaycastTheme {
    pub fn placeholder_color() -> Color {
        Color::from_rgba(1.0, 1.0, 1.0, 0.3)
    }

    pub fn selected_bg_color() -> Color {
        Color::WHITE
    }

    pub fn selected_text_color() -> Color {
        Color::BLACK
    }

    pub fn search_bg_color() -> Color {
        Color::TRANSPARENT
    }

    pub fn search_text_color() -> Color {
        Color::WHITE
    }

    pub fn search_selection_color() -> Color {
        Color::BLACK
    }

    pub fn icon_color() -> Color {
        Color::WHITE
    }
}