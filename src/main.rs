use clap::{Parser, Subcommand};
use waycast::app::AppError;
use waycast::cmd::{self, StartupError};
use waycast::core::config::{self, AppConfig};

use tracing::{error, warn};
use tracing_subscriber::{EnvFilter, fmt};

#[derive(Subcommand)]
enum Cache {
    /// Clear the cache
    Clear,
}

#[derive(Subcommand)]
enum Command {
    Version,
    /// Start the waycast daemon process
    Daemon,
    /// Signal the daemon to show the launcher UI
    Show,
    /// Show the current app configuration
    Config,
    /// Ping the daemon to check if it's up
    Status,
    /// Cache operations
    Cache {
        #[command(subcommand)]
        command: Cache,
    },
}

#[derive(Parser)]
struct App {
    #[command(subcommand)]
    command: Command,
}

pub fn main() -> Result<(), StartupError> {
    tracing_subscriber::fmt()
        .with_span_events(fmt::format::FmtSpan::CLOSE | fmt::format::FmtSpan::NEW)
        .with_env_filter(EnvFilter::new("error").add_directive("waycast=trace".parse().unwrap()))
        .init();

    let app = App::parse();

    let cfg = if config::is_development_mode() {
        AppConfig::development()
    } else {
        AppConfig::default()
    };

    // TODO: File config isn't being parsed properly
    // TODO: projects open command isn't in the AppConfig. Everything should be in the app config
    match app.command {
        Command::Version => cmd::version_command(),
        Command::Daemon => match cmd::start_daemon_command(cfg) {
            Ok(()) => Ok(()),
            Err(StartupError::ApplicationError(AppError::AlreadyRunning)) => {
                warn!("The waycast daemon is already running");
                Ok(())
            }
            Err(e) => {
                error!(%e, "Failed to start the waycast daemon");
                Err(e)
            }
        },
        Command::Status => cmd::status_command(cfg.socket_file),
        Command::Show => cmd::show_ui_command(cfg.socket_file),
        Command::Cache { command } => match command {
            Cache::Clear => cmd::cache_clear_command(cfg.database_file),
        },
        Command::Config => {
            println!("{:#?}", cfg);
            Ok(())
        }
    }
}
