use gtk::Application;
use gtk::prelude::*;
use waycast::launcher::WaycastLauncher;
use waycast::plugins;
use waycast::ui::gtk::GtkLauncherUI;

fn main() {
    let app = Application::builder()
        .application_id("dev.thegrind.waycast")
        .build();

    app.connect_activate(|app| {
        let mut file_search_plugin = plugins::file_search::new();

        match file_search_plugin.add_search_path("/home/javi/working-files/DJ Music/") {
            Err(e) => eprintln!("{}", e),
            _ => (),
        }

        // Create the core launcher
        let launcher = WaycastLauncher::new()
            .add_plugin(Box::new(plugins::drun::new()))
            .add_plugin(Box::new(file_search_plugin))
            .init();

        // Create and show the GTK UI
        let ui = GtkLauncherUI::new(app, launcher);

        // Apply built-in default styles
        if let Err(e) = ui.apply_default_css() {
            eprintln!("Warning: Could not apply default styles: {}", e);
        }

        // Optionally apply user CSS overrides
        // if let Err(_) = ui.apply_css("waycast.css") {
        //     // Silently ignore if user hasn't provided custom CSS
        // }

        ui.show();
    });

    app.run();
}
