use std::path::PathBuf;

use notify_rust::Notification;
use thiserror::Error;
use tracing::{error, info};

use crate::{
    app::{AppError, WaycastApplication},
    core::{
        config::{self, AppConfig},
        data::{DataError, WaycastData},
    },
    daemon::DaemonError,
    socket::{SocketError, WaycastSocketClient},
};

#[derive(Debug, Error)]
pub enum StartupError {
    #[error(transparent)]
    Daemon(#[from] DaemonError),
    #[error(transparent)]
    ApplicationError(#[from] AppError),
    #[error("The waycast daemon process is not running")]
    DaemonNotRunning(#[from] SocketError),
    #[error("Could not build tokio runtime")]
    TokioRuntimeFailed,
    #[error("Database error: {0}")]
    DataError(#[from] DataError),
}

pub fn show_ui_command(socket_file: PathBuf) -> Result<(), StartupError> {
    let mut client = WaycastSocketClient::new(socket_file)?;
    client.send_show()?;
    client.close();

    Ok(())
}

pub fn start_daemon_command(cfg: AppConfig) -> Result<(), StartupError> {
    // Create the app directories if needed so we don't have
    // issues later down.
    cfg.app_dir.create().expect("Failed to create the necessary XDG directories. This is fatal. Please check your desktop environment setup");

    let config_file = cfg.config_file.clone();
    config::initialize(&config_file);

    let app = WaycastApplication::new(cfg)?;

    let _ = Notification::new()
        .summary("Waycast")
        .body("Waycast started")
        .icon("dialog-information")
        .show();

    app.run().map_err(StartupError::ApplicationError)
}

pub fn version_command() -> Result<(), StartupError> {
    println!("Waycast v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

pub fn status_command(socket_file: PathBuf) -> Result<(), StartupError> {
    if let Ok(mut client) = WaycastSocketClient::new(socket_file) {
        match client.send_ping() {
            Ok(()) => println!("Waycast is running"),
            Err(e) => {
                error!(%e, "Error talking to the daemon");
            }
        };
        client.close();
    } else {
        error!("Waycast daemon is not running");
    }

    Ok(())
}

pub fn cache_clear_command(database_file: PathBuf) -> Result<(), StartupError> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| StartupError::TokioRuntimeFailed)?;

    info!("Using database file: {}", database_file.display());
    rt.block_on(async {
        let db = WaycastData::writeable_connection(database_file).await?;
        db.cache().clear().await?;

        info!("Cache cleared");
        Ok(())
    })
}
