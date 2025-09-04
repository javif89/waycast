use gtk::Application;
use gtk::prelude::*;
use std::env;
use waycast::plugins;
use waycast::ui::WaycastLauncher;

// TODO: Add an init() function to the launcher plugin spec
// that will get called when loaded. That way plugins like
// file search can index the file system on init instead
// of on the fly
fn main() {
    let app = Application::builder()
        .application_id("dev.thegrind.waycast")
        .build();

    app.connect_activate(|app| {
        let launcher = WaycastLauncher::new()
            .add_plugin(plugins::drun::DrunPlugin {})
            .add_plugin(plugins::file_search::FileSearchPlugin::new())
            .initialize(app);

        launcher.borrow().show();
    });

    app.run();
}
