use clap::{Parser, Subcommand};
use crossbeam_channel::Sender;
use notify::{Config, Error, Event, EventKind, RecommendedWatcher};
use notify_debouncer_full::{DebouncedEvent, RecommendedCache, new_debouncer_opt, notify};
use notify_rust::Notification;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;
use waycast::{FileEvent, watch_directories};
use waycast_core::WaycastScanner;
use waycast_daemon::scanners::ApplicationScanner;
use waycast_data::WaycastData;

use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt};
use waycast_daemon::WaycastDaemon;
use waycast_ui::WaycastUi;

mod watcher;

#[derive(Subcommand)]
enum Command {
    Version,
}

#[derive(Parser)]
struct App {
    #[command(subcommand)]
    command: Option<Command>,
}

pub fn main() {
    tracing_subscriber::fmt()
        .with_span_events(fmt::format::FmtSpan::CLOSE | fmt::format::FmtSpan::NEW)
        .with_env_filter(
            EnvFilter::new("error")
                .add_directive("waycast=trace".parse().unwrap())
                .add_directive("waycast_data=trace".parse().unwrap())
                .add_directive("waycast_ui=trace".parse().unwrap())
                .add_directive("waycast_daemon=trace".parse().unwrap()),
        )
        .init();

    let (tx, rx) = crossbeam_channel::unbounded::<FileEvent>();
    let app_dirs = freedesktop::application_entry_paths();

    let app_entry_thread = std::thread::spawn(move || {
        watch_directories(app_dirs, tx, notify::RecursiveMode::NonRecursive);
    });

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    std::thread::spawn(move || {
        info!("Watching for changes to application entries");
        for _ in rx {
            info!("App change happened!");
            info!("Rescanning app entries");
            rt.block_on(async move {
                let db = WaycastData::writeable_connection("waycast.db").await;
                let app_entries = ApplicationScanner::default()
                    .scan()
                    .iter()
                    .map(|i| i.to_owned().into())
                    .collect();
                let result = db
                    .items()
                    .insert_of_kind(app_entries, waycast_data::items::ItemKind::DesktopEntry)
                    .await;

                match result {
                    Ok(_) => info!("Application entry rescan successful"),
                    Err(e) => error!("Error on application entry scan: {e}"),
                }
            });
        }
    });

    let app = App::parse();

    match app.command {
        Some(Command::Version) => {
            println!("Waycast v{}", env!("CARGO_PKG_VERSION"));
        }
        None => start_waycast(),
    }
}

fn start_waycast() {
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

    let _scanner_thread_handle = std::thread::spawn(move || {
        WaycastDaemon::new().run();
    });

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
