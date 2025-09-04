use gtk::Application;
use gtk::prelude::*;
use waycast::plugins;
use waycast::ui::WaycastLauncher;

fn main() {
    let app = Application::builder()
        .application_id("dev.thegrind.waycast")
        .build();

    app.connect_activate(|app| {
        let launcher = WaycastLauncher::new()
            .add_plugin(plugins::drun::DrunPlugin {})
            .add_plugin(plugins::file_search::FileSearchPlugin {})
            .initialize(app);

        launcher.borrow().show();
    });

    app.run();
}
