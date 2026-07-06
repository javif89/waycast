mod app;
mod config;
mod styles;
mod theme;

use std::path::PathBuf;

use iced_layershell::Application;
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};

use app::Waycast;

pub struct WaycastUi;

impl WaycastUi {
    pub fn run(database_file: PathBuf) -> Result<(), iced_layershell::Error> {
        Waycast::run(Settings {
            id: Some(config::APP_NAME.into()),
            flags: database_file,
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
}
