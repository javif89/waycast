use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use notify_rust::Notification;
use thiserror::Error;
use tracing::{error, info};

use crate::{
    app::{AppError, WaycastApplication},
    core::{
        config::AppConfig,
        data::{DataError, WaycastData},
    },
    socket::{SocketError, WaycastSocketClient},
};

#[derive(Debug, Error)]
pub enum StartupError {
    #[error(transparent)]
    ApplicationError(#[from] AppError),
    #[error("The waycast daemon process is not running")]
    DaemonNotRunning(#[from] SocketError),
    #[error("Could not build tokio runtime")]
    TokioRuntimeFailed,
    #[error("Database error: {0}")]
    DataError(#[from] DataError),
    #[error("Could not render the configuration: {0}")]
    ConfigRender(#[from] toml::ser::Error),
}

pub fn config_command(cfg: &AppConfig) -> Result<(), StartupError> {
    let rendered = format!(
        "# Resolved configuration.\n\
         # This contains values from your waycast.toml as\n\
         # well as runtime resolved config values.\n\
         {}",
        toml::to_string_pretty(cfg)?
    );

    if highlight(&rendered).is_none() {
        print!("{rendered}");
    }

    Ok(())
}

/// Pipe through bat when it is installed. `None` means no bat binary was
/// found and the caller should print the plain text itself.
///
/// Debian and Ubuntu ship the binary as `batcat` because `bat` collides with
/// an ACPI tool, so both names get tried.
fn highlight(rendered: &str) -> Option<()> {
    for program in ["bat", "batcat"] {
        let Ok(mut child) = Command::new(program)
            .args([
                "--language",
                "toml",
                "--style",
                "plain",
                "--paging",
                "never",
            ])
            .stdin(Stdio::piped())
            .spawn()
        else {
            continue;
        };

        // Dropping stdin closes the pipe, which is what gives bat its EOF.
        // Holding it across wait() would deadlock.
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(rendered.as_bytes());
        }
        let _ = child.wait();

        // Committed once the spawn succeeded: a write failure here means bat
        // died early, and falling back would print the config twice.
        return Some(());
    }

    None
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
