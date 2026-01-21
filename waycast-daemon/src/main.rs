use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use directories::UserDirs;
use freedesktop::ApplicationEntry;
use gio::glib::object::Cast;
use ignore::Walk;
use tokio::time;
use tracing::{Instrument, error, info, info_span};
use tracing_subscriber::fmt;
use waycast_core::LauncherItem;
use waycast_data::{DB, DataError, ItemKind, ItemRow, wal_connection};
use waycast_plugins::{drun::DrunPlugin, file_search};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_span_events(fmt::format::FmtSpan::CLOSE | fmt::format::FmtSpan::NEW)
        .init();

    info!("Daemon starting up...");

    let db = DB::open(wal_connection("waycast.db")).await;

    let mut cadence = time::interval(Duration::from_secs(20));

    loop {
        cadence.tick().await;

        let scan_span = info_span!("scan_and_update");

        let result = scan_and_update(&db).instrument(scan_span).await;

        match result {
            Ok(_) => info!("Items inserted successfully"),
            Err(e) => error!("Error: {e}"),
        }
    }
}

async fn scan_and_update(db: &DB) -> Result<(), DataError> {
    info!("Gathering data");
    let start = Instant::now();
    let (de, f, p) = tokio::join!(
        tokio::task::spawn_blocking(|| waycast_plugins::drun::all()),
        tokio::task::spawn_blocking(|| waycast_plugins::file_search::all()),
        tokio::task::spawn_blocking(|| waycast_plugins::projects::all()),
    );

    let desktop_entries = de.unwrap_or(Vec::new());
    let files = f.unwrap_or(Vec::new());
    let projects = p.unwrap_or(Vec::new());
    let elapsed = start.elapsed();
    info!("Scan all took {:?}", elapsed);
    info!(
        "{} DE | {} Files | {} Projects",
        desktop_entries.len(),
        files.len(),
        projects.len()
    );

    let mut items: Vec<LauncherItem> =
        Vec::with_capacity(desktop_entries.len() + files.len() + projects.len());

    items.extend(desktop_entries);
    items.extend(files);
    items.extend(projects);

    info!("Inserting {} items", items.len());
    db.insert_items(items.iter().map(|i| i.to_owned().into()).collect())
        .await?;

    Ok(())
}
