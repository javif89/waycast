use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use tracing::{Instrument, error, info, info_span};
use tracing_subscriber::fmt;
use waycast_core::LauncherItem;
use waycast_data::{DB, DataError, ItemKind, ItemRow, wal_connection};
use waycast_plugins::{drun::DrunPlugin, file_search};

pub async fn scan_and_update(db: &DB) -> Result<(), DataError> {
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
