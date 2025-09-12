use waycast_core::WaycastLauncher;

fn main() {
    // Create the core launcher
    let mut launcher = WaycastLauncher::new()
        .add_plugin(Box::new(waycast_plugins::drun::new()))
        .add_plugin(Box::new(waycast_plugins::file_search::new()))
        .add_plugin(Box::new(waycast_plugins::projects::new()))
        .init();

    launcher.search("Obs");
    for r in launcher.current_results() {
        println!("ID: {} | Title: {}", r.id(), r.title());
    }
}
