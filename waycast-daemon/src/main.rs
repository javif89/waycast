use std::error::Error;
use std::sync::{Arc, Mutex};
use waycast_core::{LauncherListItem, WaycastLauncher};
use waycast_protocol::LauncherItem;
use zbus::{connection, interface};

struct WaycastService {
    launcher: Arc<Mutex<WaycastLauncher>>,
}

#[interface(name = "dev.waycast.Daemon")]
impl WaycastService {
    fn search(&self, query: &str) -> Vec<LauncherItem> {
        let mut launcher = self.launcher.lock().unwrap();
        launcher.search(query);
        launcher
            .current_results()
            .iter()
            .map(|r| LauncherItem::from(r))
            .collect()
    }

    fn default_list(&self) -> Vec<LauncherItem> {
        let mut launcher = self.launcher.lock().unwrap();
        launcher
            .get_default_results()
            .iter()
            .map(|r| LauncherItem::from(r))
            .collect()
    }

    fn execute(&self, id: &str) {
        let launcher = self.launcher.lock().unwrap();
        let _ = launcher.execute_item_by_id(id);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let launcher = WaycastLauncher::new()
        .add_plugin(Box::new(waycast_plugins::drun::new()))
        .add_plugin(Box::new(waycast_plugins::file_search::new()))
        .add_plugin(Box::new(waycast_plugins::projects::new()))
        .init();

    let service = WaycastService {
        launcher: Arc::new(Mutex::new(launcher)),
    };

    let _conn = connection::Builder::session()?
        .name("dev.waycast.Daemon")?
        .serve_at("/dev/waycast/Daemon", service)?
        .build()
        .await?;

    println!("Waycast daemon running...");
    std::future::pending::<()>().await;
    Ok(())
}
