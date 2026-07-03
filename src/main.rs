use clap::{Parser, Subcommand};
use notify_rust::Notification;
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use waycast::core::config::{self, data_dir};
use waycast::daemon::{DaemonError, scanners};

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

static DATABASE_FILENAME: &str = "waycast.db";

#[derive(Debug, Error)]
enum StartupError {
    #[error("Could not determine the Waycast data directory")]
    DataDirectoryUnavailable,

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
    let _lock = match acquire_single_instance_lock("waycast") {
        Ok(lock) => lock,
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
            info!("Another instance is already running.");
            info!("Sending show command");
            let sock = runtime_dir().join("waycast.sock");
            let mut stream = UnixStream::connect(sock).expect("Could not connect to socket");
            // TODO: We should log if there's errors with the
            // socket so the user can debug
            let _ = stream.write_all(b"show\n");
            let _ = stream.shutdown(Shutdown::Write);
            std::process::exit(0);
        }
        Err(e) => panic!("Some other non lock related error {e}"),
    };

    let daemon = create_daemon()?;
    let _scanner_thread_handle = std::thread::spawn(move || daemon.run());

    let ui_thread_handle = std::thread::spawn(move || {
        loop {
            let sock = runtime_dir().join("waycast.sock");
            let _ = std::fs::remove_file(&sock);

            let listener = UnixListener::bind(&sock).unwrap();

            info!("Waiting for signal...");
            let (mut conn, _addr) = listener.accept().unwrap();
            let mut buf = [0u8; 4096];
            let n = conn.read(&mut buf).unwrap_or(0);

            let msg = std::str::from_utf8(&buf[..n])
                .unwrap_or("<non-utf8>")
                .trim();

            if msg == "show" {
                info!("Launching UI");
                let _ = WaycastUi::run();
                info!("App exited");
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

fn lock_path(app_id: &str) -> PathBuf {
    // Prefer XDG_RUNTIME_DIR on Linux if available; else /tmp.
    // You can swap this for directories::ProjectDirs if you want.
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    base.join(format!("{app_id}.lock"))
}

pub fn acquire_single_instance_lock(app_id: &str) -> io::Result<File> {
    let path = lock_path(app_id);

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true) // file existence doesn't matter
        .open(&path)?;

    // Try to acquire an exclusive lock without blocking.
    // If this errors with "would block", another instance holds the lock.
    fs2::FileExt::try_lock_exclusive(&file)?;

    Ok(file)
}

fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

fn create_daemon() -> Result<WaycastDaemon, StartupError> {
    let project_scan_paths =
        config::get::<HashSet<PathBuf>>("plugins.projects.search_paths").unwrap_or_default();

    let (file_scan_paths, file_ignore_dirs) = get_file_search_paths();

    let database_path = data_dir()
        .ok_or(StartupError::DataDirectoryUnavailable)?
        .join(DATABASE_FILENAME);

    info!(path = %database_path.display(), "Initializing database");

    WaycastDaemon::new(
        database_path,
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
