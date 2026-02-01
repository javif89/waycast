mod app;
mod config;
mod icons;
mod theme;
mod ui;

use std::io::Read;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::time::Duration;

use iced_layershell::Application;
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};

use app::Waycast;
use tokio::time;
use tracing::{Instrument, error, info, info_span};
use tracing_subscriber::fmt;
use waycast_data::{DB, wal_connection};
use waycast_scanner::scan_and_update;

fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir())
}

pub fn main() {
    println!("Waycast v{}", env!("CARGO_PKG_VERSION"));

    tracing_subscriber::fmt()
        .with_span_events(fmt::format::FmtSpan::CLOSE | fmt::format::FmtSpan::NEW)
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Daemon starting up...");

    let scanner_thread_handle = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(4)
            .build()
            .unwrap();

        runtime.block_on(async move {
            println!("here?");
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
        });
    });

    let ui_thread_handle = std::thread::spawn(move || {
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

    // scanner_thread_handle.join().unwrap();

    // Ok(())

    // Some apps (like steam) need a bit of a grace period
    // to properly detach from the parent process. By
    // taking this approach, the UI can hide
    // immediately for good UX, but the
    // process will wait a little bit
    // before closing.
    std::thread::sleep(Duration::from_millis(1000));
}
