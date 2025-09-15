mod ui;

use gtk::Application;
use gtk::prelude::*;
use ui::gtk::GtkLauncherUI;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("WayCast v{}", env!("CARGO_PKG_VERSION"));
    
    // Connect to the daemon
    let connection = zbus::Connection::session().await?;
    let daemon_proxy = waycast_ipc::WaycastServiceProxy::new(&connection).await?;
    
    let app = Application::builder()
        .application_id("dev.thegrind.waycast")
        .build();

    let daemon_proxy_clone = daemon_proxy.clone();
    app.connect_activate(move |app| {
        // Create and show the GTK UI
        let ui = GtkLauncherUI::new(app, daemon_proxy_clone.clone());

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
    Ok(())
}
