mod app;
mod config;
mod icons;
mod theme;
mod ui;

use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::Application;

use app::Waycast;

pub fn main() -> Result<(), iced_layershell::Error> {
    Waycast::run(Settings {
        id: Some(config::APP_NAME.into()),
        layer_settings: LayerShellSettings {
            size: Some((config::WINDOW_WIDTH, config::WINDOW_HEIGHT)),
            exclusive_zone: 0,
            anchor: Anchor::Bottom | Anchor::Left | Anchor::Right | Anchor::Top,
            start_mode: StartMode::Active,
            ..Default::default()
        },
        ..Default::default()
    })
}