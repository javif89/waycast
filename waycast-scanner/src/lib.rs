use std::{
    collections::HashSet,
    path::PathBuf,
    time::Instant,
};

use tracing::{Instrument, info, info_span};
use waycast_core::{LauncherItem, WaycastScanner};
use waycast_data::{DB, DataError};
use waycast_plugins::{
    drun::ApplicationScanner,
    file_search::{self, FileScanner},
    projects::ProjectScanner,
};

pub async fn scan_and_update(db: &DB) -> Result<(), DataError> {
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
    db.insert_items(items.iter().map(|i| i.to_owned().into()).collect())
        .instrument(insert_span)
        .await?;

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
        search_paths.extend(file_search::default_search_list());
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
