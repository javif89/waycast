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
        // Create the core launcher
        let launcher = WaycastLauncher::new()
            .add_plugin(Box::new(plugins::drun::DrunPlugin {}))
            .add_plugin(Box::new(plugins::file_search::FileSearchPlugin::new()))
            .init();

        // Create and show the GTK UI
        let ui = GtkLauncherUI::new(app, launcher);
        ui.show();
    });

    app.run();
}
