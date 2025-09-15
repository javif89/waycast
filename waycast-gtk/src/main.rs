mod ui;

use gtk::Application;
use gtk::prelude::*;
use ui::gtk::GtkLauncherUI;
use waycast_protocol::WaycastClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("WayCast v{}", env!("CARGO_PKG_VERSION"));
    
    let app = Application::builder()
        .application_id("dev.thegrind.waycast")
        .build();

    app.connect_activate(|app| {
        // Connect to the daemon synchronously
        match WaycastClient::connect() {
            Ok(client) => {
                // Create and show the GTK UI
                let ui = GtkLauncherUI::new(app, client);

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
            }
            Err(e) => {
                eprintln!("Failed to connect to daemon: {}", e);
                app.quit();
            }
        }
    });

    app.run();
    Ok(())
}
