mod ui;
mod util;

use gtk::prelude::*;
use gtk::Application;
use ui::gtk::GtkLauncherUI;
use waycast_core::WaycastLauncher;

fn main() {
    let app = Application::builder()
        .application_id("dev.thegrind.waycast")
        .build();

    app.connect_activate(|app| {
        // Create the core launcher
        let launcher = WaycastLauncher::new()
            .add_plugin(Box::new(waycast_plugins::drun::new()))
            .add_plugin(Box::new(waycast_plugins::file_search::new()))
            .add_plugin(Box::new(waycast_plugins::projects::new()))
            .init();

        // Create and show the GTK UI
        let ui = GtkLauncherUI::new(app, launcher);

        // Apply built-in default styles
        if let Err(e) = ui.apply_default_css() {
            eprintln!("Warning: Could not apply default styles: {}", e);
        }

        if let Some(path) = waycast_config::config_path("waycast.css") {
            if ui.apply_css(path).is_err() {
                println!("No user css found");
            }
        }

        ui.show();
    });

    app.run();
}
