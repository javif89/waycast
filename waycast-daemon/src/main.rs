use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use waycast_core::WaycastLauncher;
use waycast_protocol::{LauncherItem, RequestHandler, WaycastServer, socket::default_socket_path};

struct LauncherHandler {
    launcher: Arc<Mutex<WaycastLauncher>>,
}

impl LauncherHandler {
    fn new(launcher: WaycastLauncher) -> Self {
        Self {
            launcher: Arc::new(Mutex::new(launcher)),
        }
    }
}

#[waycast_protocol::async_trait::async_trait]
impl RequestHandler for LauncherHandler {
    async fn search(&self, query: &str) -> Result<Vec<LauncherItem>, String> {
        let mut launcher = self
            .launcher
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let results = launcher.search(query);

        let items = results
            .iter()
            .map(|item| LauncherItem {
                id: item.id(),
                title: item.title(),
                description: item.description(),
                icon: item.icon(),
            })
            .collect();

        Ok(items)
    }

    async fn default_list(&self) -> Result<Vec<LauncherItem>, String> {
        let mut launcher = self
            .launcher
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let results = launcher.get_default_results();

        let items = results
            .iter()
            .map(|item| LauncherItem {
                id: item.id(),
                title: item.title(),
                description: item.description(),
                icon: item.icon(),
            })
            .collect();

        Ok(items)
    }

    async fn execute(&self, id: &str) -> Result<(), String> {
        let launcher = self
            .launcher
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        launcher
            .execute_item_by_id(id)
            .map_err(|e| format!("Execute error: {:?}", e))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let launcher = WaycastLauncher::new()
        .add_plugin(Box::new(waycast_plugins::drun::new()))
        .add_plugin(Box::new(waycast_plugins::file_search::new()))
        .add_plugin(Box::new(waycast_plugins::projects::new()))
        .init();

    let handler = Arc::new(LauncherHandler::new(launcher));
    
    // Clone the launcher reference for the refresh task
    let launcher_for_refresh = Arc::clone(&handler.launcher);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(120)); // 2 minutes
        loop {
            interval.tick().await;
            if let Ok(launcher) = launcher_for_refresh.lock() {
                println!("Refreshing plugins...");
                launcher.refresh_plugins();
            }
        }
    });
    
    let socket_path = default_socket_path()?;
    let server = WaycastServer::new(&socket_path)?;

    println!("Waycast daemon starting on {}", socket_path.display());
    server.serve(handler).await?;

    Ok(())
}
