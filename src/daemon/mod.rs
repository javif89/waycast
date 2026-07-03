use std::path::Path;
use std::sync::Arc;
use std::{collections::HashSet, path::PathBuf, time::Instant};
use thiserror::Error;

use crate::core::data::{DataError, WaycastData};
use crate::core::{LauncherItem, WaycastScanner};
use crate::daemon::watcher::{FileEvent, watch_directories};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;
use tracing::{Instrument, error, info, info_span};

pub mod scanners;
pub mod watcher;
use scanners::{ApplicationScanner, FileScanner, projects::ProjectScanner};

static MAX_MPSC_BUF_SIZE: usize = 1;

#[derive(Debug, Error)]
pub enum DaemonError {
    #[error("Failed to initialize daemon runtime: {0}")]
    Runtime(#[source] std::io::Error),

    #[error(transparent)]
    Data(#[from] DataError),
}

pub struct WaycastDaemon {
    db: WaycastData,
    rt: tokio::runtime::Runtime,
    app_scanner: Arc<ApplicationScanner>,
    project_scanner: Arc<ProjectScanner>,
    file_scanner: Arc<FileScanner>,
}

impl WaycastDaemon {
    pub fn new(
        database_path: impl AsRef<Path>,
        project_scan_paths: HashSet<PathBuf>,
        file_scan_paths: HashSet<PathBuf>,
        file_ignore_dirs: HashSet<String>,
    ) -> Result<Self, DaemonError> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(4)
            .build()
            .map_err(DaemonError::Runtime)?;

        let db = rt.block_on(WaycastData::writeable_connection(&database_path))?;
        let app_scanner = Arc::new(ApplicationScanner);
        let project_scanner = Arc::new(ProjectScanner::new(project_scan_paths));
        let file_scanner = Arc::new(FileScanner::new(file_scan_paths, file_ignore_dirs));

        Ok(Self {
            db,
            rt,
            app_scanner,
            project_scanner,
            file_scanner,
        })
    }
}

impl WaycastDaemon {
    pub fn run(&self) {
        let (app_event_tx, mut app_event_rx) = mpsc::channel(MAX_MPSC_BUF_SIZE);
        let (project_event_tx, mut project_event_rx) = mpsc::channel(MAX_MPSC_BUF_SIZE);
        let _app_watcher_handle = self.watch_app_directories(app_event_tx);
        let _project_watcher_handle = self.watch_project_directories(project_event_tx);

        self.rt.block_on(async move {
            let mut cadence = time::interval(Duration::from_secs(20));
            let mut app_watcher_open = true;
            let mut project_watcher_open = true;

            loop {
                tokio::select! {
                    _ = cadence.tick() => {
                        let scan_span = info_span!("scan_and_update");
                        let icon_cache_span = info_span!("update_icon_cache");

                        match self.scan_and_update().instrument(scan_span).await {
                            Ok(()) => {
                                info!("Items inserted successfully");
                                info!("Updating icon cache");
                                if let Err(e) = self.update_icon_cache().instrument(icon_cache_span).await {
                                    error!("Error updating icon cache {e}");
                                }
                            }
                            Err(e) => error!("Error: {e}"),
                        }
                    },
                    app_event = app_event_rx.recv(), if app_watcher_open => {
                        match app_event {
                            Some(FileEvent::ChangeInDirectory) => {
                                let scan_span = info_span!("scan_and_update_apps");
                                if let Err(e) = self.scan_and_update_apps().instrument(scan_span).await {
                                    error!("Error updating application entries: {e}");
                                }
                            }
                            None => {
                                error!("Application directory watcher stopped");
                                app_watcher_open = false;
                            }
                        }
                    },
                    projects_event = project_event_rx.recv(), if project_watcher_open => {
                        match projects_event {
                            Some(FileEvent::ChangeInDirectory) => {
                                let scan_span = info_span!("scan_and_update_projects");
                                if let Err(e) = self.scan_and_update_projects().instrument(scan_span).await {
                                    error!("Error updating projects entries: {e}");
                                }
                            }
                            None => {
                                error!("Projects directory watcher stopped");
                                project_watcher_open = false;
                            }
                        }
                    },
                }
            }
        });
    }

    fn watch_app_directories(
        &self,
        event_tx: mpsc::Sender<FileEvent>,
    ) -> std::thread::JoinHandle<()> {
        let app_dirs = freedesktop::application_entry_paths();

        std::thread::spawn(move || {
            info!("Watching for changes to application entries");
            watch_directories(app_dirs, event_tx, notify::RecursiveMode::NonRecursive);
        })
    }

    fn watch_project_directories(
        &self,
        event_tx: mpsc::Sender<FileEvent>,
    ) -> std::thread::JoinHandle<()> {
        let projects_dirs = self
            .project_scanner
            .get_search_paths()
            .into_iter()
            .collect::<Vec<PathBuf>>();

        std::thread::spawn(move || {
            info!("Watching for changes to projects entries");
            watch_directories(projects_dirs, event_tx, notify::RecursiveMode::NonRecursive);
        })
    }

    async fn scan_and_update_apps(&self) -> Result<(), DataError> {
        info!("Application directory changed; rescanning application entries");

        let scanner = Arc::clone(&self.app_scanner);
        let app_entries = tokio::task::spawn_blocking(move || scanner.scan())
            .await
            .map_err(|e| DataError::QueryError(format!("Application scanner task failed: {e}")))?;

        self.db
            .items()
            .insert_of_kind(app_entries, crate::core::ItemKind::DesktopEntry)
            .await?;

        info!("Application entry rescan successful; updating icon cache");
        self.update_icon_cache().await?;

        Ok(())
    }

    async fn scan_and_update_projects(&self) -> Result<(), DataError> {
        info!("Projects directory changed; rescanning projects entries");

        let scanner = Arc::clone(&self.project_scanner);
        let project_entries = tokio::task::spawn_blocking(move || scanner.scan())
            .await
            .map_err(|e| DataError::QueryError(format!("Projects scanner task failed: {e}")))?;

        self.db
            .items()
            .insert_of_kind(project_entries, crate::core::ItemKind::Project)
            .await?;

        info!("Projects entry rescan successful");

        Ok(())
    }

    async fn scan_and_update(&self) -> Result<(), DataError> {
        info!("Gathering data");
        let start = Instant::now();

        let s_app = self.app_scanner.clone();
        let s_projects = self.project_scanner.clone();
        let s_files = self.file_scanner.clone();
        let (de, f, p) = tokio::join!(
            tokio::task::spawn_blocking(move || s_app.scan()),
            tokio::task::spawn_blocking(move || s_files.scan()),
            tokio::task::spawn_blocking(move || s_projects.scan()),
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
        self.db
            .items()
            .insert(items)
            .instrument(insert_span)
            .await?;

        Ok(())
    }

    /// Warm the icon cache so that we ideally only get cache hits in the UI
    async fn update_icon_cache(&self) -> Result<(), DataError> {
        info!("Warming icon cache");
        let icons: Vec<String> = self.db.items().get_icons().await.unwrap_or(Vec::new());

        for i in icons {
            // Check if it's a path. If not, then it's
            // a themed icon. We will resolve its
            // path and cache it.

            let path = std::path::Path::new(&i);
            if !path.exists() {
                let key = format!("icon:{}", i);
                if let Some(icon_path) = freedesktop::get_icon(&i) {
                    self.db
                        .cache()
                        .put(&key, &icon_path, Some(Duration::from_hours(8)))
                        .await?;
                }
            }
        }

        Ok(())
    }
}
