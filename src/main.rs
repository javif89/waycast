use clap::{Parser, Subcommand};
use notify_rust::Notification;
use waycast::app::{AppError, WaycastApplication};
use waycast::core::config::{self, AppConfig};
use waycast::daemon::DaemonError;
use waycast::socket::WaycastSocketCient;

use thiserror::Error;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt};

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
    #[error(transparent)]
    ApplicationEror(#[from] AppError),
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

    // TODO: Fix some ownership issues here so
    // we don't have to clone individual
    // config fields. Won't be an issue if
    // we just have a dedicated "show" command
    // instead of doing this check lol.
    let socket_file = cfg.socket_file.clone();
    let config_file = cfg.config_file.clone();
    let app = match WaycastApplication::new(cfg) {
        Ok(app) => app,
        Err(e) => match e {
            AppError::AlreadyRunning => {
                info!("Another instance is already running.");
                info!("Sending show command");
                let mut client = WaycastSocketCient::new(socket_file);
                client.send_show();
                client.close();
                std::process::exit(0);
            }
            _ => panic!("Critical error: {}", e),
        },
    };

    config::initialize(&config_file);

    let _ = Notification::new()
        .summary("Waycast")
        .body("Waycast started")
        .icon("dialog-information")
        .show();

    app.run().map_err(StartupError::ApplicationEror)
}
