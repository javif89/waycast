use clap::{Parser, Subcommand};
use notify_rust::Notification;
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::PathBuf;
use waycast::core::config::{self, AppConfig};
use waycast::daemon::{DaemonError, scanners};
use waycast::socket::{SocketCommand, WaycastSocketCient, WaycastSocketListener};

use thiserror::Error;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt};
use waycast::daemon::WaycastDaemon;
use waycast::ui::WaycastUi;

#[derive(Subcommand)]
enum Command {
    Version,
}

#[derive(Parser)]
struct App {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Error)]
enum StartupError {
    #[error(transparent)]
    Daemon(#[from] DaemonError),
}

pub fn main() {
    tracing_subscriber::fmt()
        .with_span_events(fmt::format::FmtSpan::CLOSE | fmt::format::FmtSpan::NEW)
        .with_env_filter(EnvFilter::new("error").add_directive("waycast=trace".parse().unwrap()))
        .init();

    let app = App::parse();

    match app.command {
        Some(Command::Version) => {
            println!("Waycast v{}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            if let Err(error) = start_waycast() {
                error!(%error, "Failed to start Waycast");
                std::process::exit(1);
            }
        }
    }
}

fn start_waycast() -> Result<(), StartupError> {
    let cfg = if config::is_development_mode() {
        AppConfig::development()
    } else {
        AppConfig::default()
    };

    // Create all the necessary app directories in case they don't exist
    // TODO: Move this after the lock acquisition. I don't want anything
    // adding latency to the UI startup command being sent.
    // This will also be solved when we put showing the UI on its own
    // command rather than relying on the lock to either start the daemon
    // or show the UI.
    cfg.app_dir.create().expect("Failed to create the necessary XDG directories. This is fatal. Please check your desktop environment setup");

    let _lock = match acquire_single_instance_lock(&cfg) {
        Ok(lock) => lock,
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
            info!("Another instance is already running.");
            info!("Sending show command");
            let mut client = WaycastSocketCient::new(cfg.socket_file.clone());
            client.send_show();
            client.close();
            std::process::exit(0);
        }
        Err(e) => panic!("Some other non lock related error {e}"),
    };

    config::initialize(&cfg.config_file);

    let daemon = create_daemon(&cfg)?;
    let _scanner_thread_handle = std::thread::spawn(move || daemon.run());

    let ui_thread_handle = std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel::<SocketCommand>();
        let listener = WaycastSocketListener::new(cfg.socket_file.clone());

        std::thread::spawn(move || {
            listener.listen(tx);
        });

        for cmd in rx {
            match cmd {
                SocketCommand::Show => {
                    info!("Launching UI");
                    let _ = WaycastUi::run(cfg.database_file.clone());
                    info!("App exited");
                }
            }
        }
    });

    let _ = Notification::new()
        .summary("Waycast")
        .body("Daemon started successfully")
        .icon("dialog-information")
        .show();

    ui_thread_handle.join().unwrap();
    Ok(())
}

pub fn acquire_single_instance_lock(cfg: &AppConfig) -> io::Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true) // file existence doesn't matter
        .open(cfg.lock_file.clone())?;

    // Try to acquire an exclusive lock without blocking.
    // If this errors with "would block", another instance holds the lock.
    fs2::FileExt::try_lock_exclusive(&file)?;

    Ok(file)
}

fn create_daemon(cfg: &AppConfig) -> Result<WaycastDaemon, StartupError> {
    let project_scan_paths =
        config::get::<HashSet<PathBuf>>("plugins.projects.search_paths").unwrap_or_default();

    let (file_scan_paths, file_ignore_dirs) = get_file_search_paths();

    info!(path = %cfg.database_file.display(), "Initializing database");

    WaycastDaemon::new(
        &cfg.database_file,
        project_scan_paths,
        file_scan_paths,
        file_ignore_dirs,
    )
    .map_err(Into::into)
}

fn get_file_search_paths() -> (HashSet<PathBuf>, HashSet<String>) {
    // Gather file scanning paths and dirs to ignore
    let mut search_paths: HashSet<PathBuf> = HashSet::new();
    let mut skip_dirs: HashSet<String> = HashSet::new();

    if let Ok(paths) = config::config_file().get::<Vec<PathBuf>>("plugins.file_search.search_paths")
    {
        search_paths.extend(paths);
    } else {
        search_paths.extend(scanners::default_search_list());
    }

    if let Ok(paths) = config::config_file().get::<Vec<String>>("plugins.file_search.ignore_dirs") {
        skip_dirs.extend(paths);
    }

    (search_paths, skip_dirs)
}
