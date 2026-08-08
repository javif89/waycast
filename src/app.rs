use std::{
    fs::OpenOptions,
    sync::Arc,
    sync::mpsc::{Receiver, Sender},
};

use thiserror::Error;
use tracing::info;

use crate::{
    core::config::AppConfig,
    daemon::WaycastDaemon,
    facade::{self, WaycastFacade},
    socket::WaycastSocketListener,
    ui::WaycastUi,
};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Another waycast instance is already running")]
    AlreadyRunning,
    #[error("Could not create lockfile: {0}")]
    LockIO(#[source] std::io::Error),
    #[error("Failed to initialize runtime: {0}")]
    Runtime(#[source] std::io::Error),
    #[error("Daemon thread panicked")]
    DaemonPanic,
    #[error(transparent)]
    Facade(#[from] facade::WaycastError),
}

/// Main container for the waycast app process. This acts
/// as a central way to thread configs down to the pieces
/// that need them as well as acting as a process suppervisor
/// and central message bridge between threads.
pub struct WaycastApplication {
    waycast: Arc<WaycastFacade>,
    // TODO: We can definitely merge daemon functionality into the general
    // app functionality. But doing it this way for now while we clean
    // the other stuff up.
    daemon: WaycastDaemon,
    // Message channel for thread communication
    message_channel: Receiver<AppMessage>,
    // Message sender to be cloned and passed to the threads that need it.
    message_sender: Sender<AppMessage>,
    // Listener for IPC messages
    socket_listener: WaycastSocketListener,
    // Hold the lockfile so we can make sure only
    // one instance of waycast is running.
    _lock: std::fs::File,
}

#[derive(Debug)]
pub enum AppMessage {
    Show,
    /// Ping the daemon and check if it's running
    Ping,
    Rescan,
    Stop,
}

impl WaycastApplication {
    pub fn new(cfg: AppConfig) -> Result<Self, AppError> {
        let lockfile = Self::get_lock(&cfg)?;

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(4)
            .build()
            .map_err(AppError::Runtime)?;

        let (message_sender, message_channel) = std::sync::mpsc::channel::<AppMessage>();

        let socket_listener = WaycastSocketListener::new(cfg.socket_file.clone());

        let waycast = Arc::new(WaycastFacade::new(cfg, rt.handle().clone())?);
        let daemon = WaycastDaemon::new(waycast.clone(), rt);

        Ok(Self {
            waycast,
            daemon,
            message_channel,
            message_sender,
            socket_listener,
            _lock: lockfile,
        })
    }

    /// Starts up all our little threads.
    /// Important note: Currently the only thread we join is the daemon
    /// thread. If the daemon fails then we just shut everything down.
    /// This isn't good or bad, but I would like to have some better
    /// failure recovery modes in the future.
    pub fn run(self) -> Result<(), AppError> {
        let daemon_handle = std::thread::spawn(move || self.daemon.run());

        let socket_sender_clone = self.message_sender.clone();
        let _socket_listener_handle = std::thread::spawn(move || {
            info!("Starting IPC message listener");
            self.socket_listener.listen(socket_sender_clone);
        });

        // Central place to listen and act on app messages
        let _message_listener_thread = std::thread::spawn(move || {
            info!("Starting app message central listener");
            for cmd in &self.message_channel {
                info!("Received app message {:#?}", cmd);
                match cmd {
                    AppMessage::Show => {
                        Self::show_ui(self.waycast.clone());
                    }
                    AppMessage::Ping => {
                        info!("Received ping");
                    }
                    AppMessage::Rescan => todo!(),
                    AppMessage::Stop => todo!(),
                }
            }
        });

        match daemon_handle.join() {
            Ok(_) => Ok(()),
            Err(_) => Err(AppError::DaemonPanic),
        }
    }

    // NOTE: It is assumed that this will be called from within
    // the long running waycast process. If this were to
    // be called normally, the thread would just be
    // immediately dropped and not do anything.
    fn show_ui(waycast: Arc<WaycastFacade>) {
        std::thread::spawn(move || {
            info!("Launching UI");
            let _ = WaycastUi::run(waycast);
            info!("App exited");
        });
    }

    /// Get the single instance lock to make sure we don't start
    /// two daemon processes.
    /// An error creating the file does not mean another instance is
    /// running. Only try_lock_exclusive returning Err::WouldBlock
    /// means another instance is running.
    fn get_lock(cfg: &AppConfig) -> Result<std::fs::File, AppError> {
        tracing::info!("Attempting to get lock {}", cfg.lock_file.display());
        let lockfile = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true) // file existence doesn't matter
            .truncate(true)
            .open(cfg.lock_file.clone())
            .map_err(AppError::LockIO)?;

        match fs2::FileExt::try_lock_exclusive(&lockfile) {
            Ok(()) => Ok(lockfile),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Err(AppError::AlreadyRunning),
            Err(e) => Err(AppError::LockIO(e)),
        }
    }
}
