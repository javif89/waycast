mod app;
mod config;
mod styles;
mod theme;

use std::sync::Arc;

use iced_layershell::Application;
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};

use crate::facade::WaycastFacade;
use app::Waycast;

pub struct WaycastUi;

impl WaycastUi {
    pub fn run(waycast: Arc<WaycastFacade>) -> Result<(), iced_layershell::Error> {
        // `Settings` only derives `Default` when `Flags: Default`, and the
        // facade has no meaningful default, so borrow the defaults for
        // every other field from a unit-flagged `Settings`.
        let defaults = Settings::<()>::default();

        Waycast::run(Settings {
            id: Some(config::APP_NAME.into()),
            flags: waycast,
            layer_settings: LayerShellSettings {
                size: Some((config::WINDOW_WIDTH, config::WINDOW_HEIGHT)),
                exclusive_zone: 0,
                anchor: Anchor::Bottom | Anchor::Left | Anchor::Right | Anchor::Top,
                start_mode: StartMode::Active,
                ..Default::default()
            },
            antialiasing: defaults.antialiasing,
            default_font: defaults.default_font,
            default_text_size: defaults.default_text_size,
            fonts: defaults.fonts,
            virtual_keyboard_support: defaults.virtual_keyboard_support,
        })
    }
}
