use std::{collections::HashSet, path::PathBuf, time::Instant};

use std::time::Duration;
use tokio::time;
use tracing::{Instrument, error, info, info_span};
use waycast_core::{LauncherItem, WaycastScanner};
use waycast_data::{DataError, WaycastData, icons::IconRow};
use waycast_facade::{Icon, WaycastLauncher};
mod scanners;
use scanners::{ApplicationScanner, FileScanner, projects::ProjectScanner};

use crate::scanners::default_search_list;

pub struct WaycastDaemon {}

impl Default for WaycastDaemon {
    fn default() -> Self {
        Self::new()
    }
}

impl WaycastDaemon {
    pub fn new() -> Self {
        Self {}
    }
}

impl WaycastDaemon {
    pub fn run(&self) {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(4)
            .build()
            .unwrap();

        runtime.block_on(async move {
            let db = WaycastData::writeable_connection("waycast.db").await;
            let mut cadence = time::interval(Duration::from_secs(20));

            loop {
                cadence.tick().await;

                let scan_span = info_span!("scan_and_update");
                let icon_cache_span = info_span!("update_icon_cache");

                let result = scan_and_update(&db).instrument(scan_span).await;

                match result {
                    Ok(_) => {
                        info!("Items inserted successfully");
                        info!("Updating icon cache");
                        if let Err(e) = update_icon_cache(&db).instrument(icon_cache_span).await {
                            error!("Error updating icon cache {e}");
                        }
                    }
                    Err(e) => error!("Error: {e}"),
                }
            }
        });
    }
}

async fn scan_and_update(db: &WaycastData) -> Result<(), DataError> {
    info!("Gathering data");
    let start = Instant::now();

    let (de, f, p) = tokio::join!(
        tokio::task::spawn_blocking(|| ApplicationScanner.scan()),
        tokio::task::spawn_blocking(|| init_file_scanner().scan()),
        tokio::task::spawn_blocking(|| init_project_scanner().scan()),
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
    let insert_span = info_span!("inserting");
    db.items()
        .insert(items.iter().map(|i| i.to_owned().into()).collect())
        .instrument(insert_span)
        .await?;

    Ok(())
}

/// Warm the icon cache so that we ideally only get cache hits in the UI
async fn update_icon_cache(db: &WaycastData) -> Result<(), DataError> {
    info!("Warming icon cache");
    let icons: Vec<String> = db.items().get_icons().await.unwrap_or(Vec::new());

    for i in icons {
        // Check if it's a path. If not, then it's
        // a themed icon. We will resolve its
        // path and cache it.

        let path = std::path::Path::new(&i);
        if !path.exists() {
            let key = format!("icon:{}", i);
            if let Some(icon_path) = freedesktop::get_icon(&i) {
                db.cache()
                    .put(&key, &icon_path, Some(Duration::from_hours(8)))
                    .await?;
            }
        }
    }

    Ok(())
}

fn init_project_scanner() -> ProjectScanner {
    let search_paths = waycast_config::get::<HashSet<PathBuf>>("plugins.projects.search_paths")
        .unwrap_or_default();

    let skip_dirs =
        waycast_config::get::<HashSet<String>>("plugins.projects.skip_dirs").unwrap_or_default();

    ProjectScanner::new().with_search_paths(search_paths)
}

fn init_file_scanner() -> FileScanner {
    // Gather file scanning paths and dirs to ignore
    let mut search_paths: HashSet<PathBuf> = HashSet::new();
    let mut skip_dirs: HashSet<String> = HashSet::new();

    if let Ok(paths) =
        waycast_config::config_file().get::<Vec<PathBuf>>("plugins.file_search.search_paths")
    {
        search_paths.extend(paths);
    } else {
        search_paths.extend(default_search_list());
    }

    if let Ok(paths) =
        waycast_config::config_file().get::<Vec<String>>("plugins.file_search.ignore_dirs")
    {
        skip_dirs.extend(paths);
    }

    FileScanner::new()
        .with_paths(search_paths)
        .with_ignore_dirs(skip_dirs)
}
