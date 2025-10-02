mod app;
mod config;
mod icons;
mod theme;
mod ui;

use std::time::Duration;

use iced_layershell::Application;
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};

use app::Waycast;

pub fn main() -> Result<(), iced_layershell::Error> {
    let result = Waycast::run(Settings {
        id: Some(config::APP_NAME.into()),
        layer_settings: LayerShellSettings {
            size: Some((config::WINDOW_WIDTH, config::WINDOW_HEIGHT)),
            exclusive_zone: 0,
            anchor: Anchor::Bottom | Anchor::Left | Anchor::Right | Anchor::Top,
            start_mode: StartMode::Active,
            ..Default::default()
        },
        ..Default::default()
    });

    // Some apps (like steam) need a bit of a grace period
    // to properly detach from the parent process. By
    // taking this approach, the UI can hide
    // immediately for good UX, but the
    // process will wait a little bit
    // before closing.
    std::thread::sleep(Duration::from_millis(1000));

    result
}
