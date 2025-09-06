use std::time::Duration;

use waycast_core::cache::Cache;

fn main() {
    let cache: Cache;
    if let Ok(db) = waycast_core::cache::new("waycast_cache") {
        cache = db;
    } else {
        panic!("Failed to open database");
    }

    let result = cache.remember_with_ttl("test_key", Some(Duration::from_secs(10)), || {
        String::from("my cool string value")
    });

    if let Ok(val) = result {
        println!("{}", val);
    }
}
// mod ui;
// mod util;

// use gtk::prelude::*;
// use gtk::Application;
// use ui::gtk::GtkLauncherUI;
// use waycast_core::WaycastLauncher;

// fn main() {
//     let app = Application::builder()
//         .application_id("dev.thegrind.waycast")
//         .build();

//     app.connect_activate(|app| {
//         let mut file_search_plugin = waycast_plugins::file_search::new();

//         if let Err(e) = file_search_plugin.add_search_path("/home/javi/working-files/DJ Music/") {
//             eprintln!("{}", e)
//         }

//         let mut project_plugin = waycast_plugins::projects::new();
//         if let Err(e) = project_plugin.add_search_path("/home/javi/projects") {
//             eprintln!("{}", e)
//         }

//         // Create the core launcher
//         let launcher = WaycastLauncher::new()
//             .add_plugin(Box::new(waycast_plugins::drun::new()))
//             .add_plugin(Box::new(file_search_plugin))
//             .add_plugin(Box::new(project_plugin))
//             .init();

//         // Create and show the GTK UI
//         let ui = GtkLauncherUI::new(app, launcher);

//         // Apply built-in default styles
//         if let Err(e) = ui.apply_default_css() {
//             eprintln!("Warning: Could not apply default styles: {}", e);
//         }

//         // Optionally apply user CSS overrides
//         // if let Err(_) = ui.apply_css("waycast.css") {
//         //     // Silently ignore if user hasn't provided custom CSS
//         // }

//         ui.show();
//     });

//     app.run();
// }
