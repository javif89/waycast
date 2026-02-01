use std::path::PathBuf;
use std::time::Duration;

use tokio::time;
use tracing::{Instrument, error, info, info_span};
use tracing_subscriber::fmt;
use waycast_data::{DB, wal_connection};
use waycast_scanner::scan_and_update;

fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

pub fn main() {
    tracing_subscriber::fmt()
        .with_span_events(fmt::format::FmtSpan::CLOSE | fmt::format::FmtSpan::NEW)
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Daemon starting up...");
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
}
