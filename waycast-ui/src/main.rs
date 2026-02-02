mod app;
mod config;
mod theme;
mod ui;

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::time::Duration;

use iced_layershell::Application;
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};

use app::Waycast;
use tracing::info;
use tracing_subscriber::fmt;
use waycast_daemon::WaycastDaemon;

fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

pub fn main() {
    println!("Waycast v{}", env!("CARGO_PKG_VERSION"));

    let _lock = match acquire_single_instance_lock("waycast") {
        Ok(lock) => lock,
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
            println!("Another instance is already running.");
            println!("Sending show command");
            let sock = runtime_dir().join("waycast.sock");
            let mut stream = UnixStream::connect(sock).expect("Could not connect to socket");
            stream.write_all(b"show\n");
            stream.shutdown(Shutdown::Write);
            std::process::exit(0);
        }
        Err(e) => panic!("Some other non lock related error {e}"),
    };

    tracing_subscriber::fmt()
        .with_span_events(fmt::format::FmtSpan::CLOSE | fmt::format::FmtSpan::NEW)
        .init();

    info!("Daemon starting up...");

    let scanner_thread_handle = std::thread::spawn(move || {
        WaycastDaemon::new().run();
    });

    let ui_thread_handle = std::thread::spawn(move || {
        // let _ = Waycast::run(Settings {
        //     id: Some(config::APP_NAME.into()),
        //     layer_settings: LayerShellSettings {
        //         size: Some((config::WINDOW_WIDTH, config::WINDOW_HEIGHT)),
        //         exclusive_zone: 0,
        //         anchor: Anchor::Bottom | Anchor::Left | Anchor::Right | Anchor::Top,
        //         start_mode: StartMode::Active,
        //         ..Default::default()
        //     },
        //     ..Default::default()
        // });
        // println!("App exited");
        loop {
            let sock = runtime_dir().join("waycast.sock");
            let _ = std::fs::remove_file(&sock);

            let listener = UnixListener::bind(&sock).unwrap();

            println!("Waiting for signal...");
            let (mut conn, _addr) = listener.accept().unwrap();
            let mut buf = [0u8; 4096];
            let n = conn.read(&mut buf).unwrap_or(0);

            let msg = std::str::from_utf8(&buf[..n])
                .unwrap_or("<non-utf8>")
                .trim();

            if msg == "show" {
                println!("Launching UI");
                let _ = Waycast::run(Settings {
                    id: Some(config::APP_NAME.into()),
                    layer_settings: LayerShellSettings {
                        size: Some((config::WINDOW_WIDTH, config::WINDOW_HEIGHT)),
                        exclusive_zone: 0,
                        anchor: Anchor::Bottom | Anchor::Left | Anchor::Right | Anchor::Top,
                        start_mode: StartMode::Active,
                        ..Default::default()
                    },
                    ..Default::default()
                });
                println!("App exited");
            }
        }
    });

    ui_thread_handle.join().unwrap();

    // Some apps (like steam) need a bit of a grace period
    // to properly detach from the parent process. By
    // taking this approach, the UI can hide
    // immediately for good UX, but the
    // process will wait a little bit
    // before closing.
    std::thread::sleep(Duration::from_millis(1000));
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
        .create(true) // file existence doesn't matter
        .open(&path)?;

    // Try to acquire an exclusive lock without blocking.
    // If this errors with "would block", another instance holds the lock.
    fs2::FileExt::try_lock_exclusive(&file)?;

    Ok(file)
}
