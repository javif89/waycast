use std::error::Error;
use std::sync::{Arc, Mutex};
use waycast_core::WaycastLauncher;
use waycast_ipc::{Client, Response, WaycastService};
use zbus::{connection, interface};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let launcher = WaycastLauncher::new()
        .add_plugin(Box::new(waycast_plugins::drun::new()))
        .add_plugin(Box::new(waycast_plugins::file_search::new()))
        .add_plugin(Box::new(waycast_plugins::projects::new()))
        .init();

    let service = WaycastService::new(launcher);

    let _conn = connection::Builder::session()?
        .name("dev.waycast.Daemon")?
        .serve_at("/dev/waycast/Daemon", service)?
        .build()
        .await?;

    println!("Waycast daemon running...");
    std::future::pending::<()>().await;
    Ok(())
}
