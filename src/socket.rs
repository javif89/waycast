use std::io::{self, BufRead, BufReader, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::sync::mpsc::Sender;
use std::{os::unix::net::UnixListener, path::PathBuf};

use thiserror::Error;
use tracing::{error, info};

use crate::app::AppMessage;

#[derive(Debug, Error)]
pub enum SocketError {
    #[error("The waycast daemon is not running")]
    DaemonNotAvailable,
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error(transparent)]
    IOError(#[from] io::Error),
}

pub struct WaycastSocketListener {
    socket_path: PathBuf,
    listener: UnixListener,
}

impl WaycastSocketListener {
    pub fn new(socket_path: PathBuf) -> Self {
        let _ = std::fs::remove_file(&socket_path);

        if let Some(parent) = socket_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let listener = UnixListener::bind(&socket_path).unwrap();

        Self {
            socket_path,
            listener,
        }
    }

    /// BLOCKS and waits for events to come through
    pub fn listen(&self, command_tx: Sender<AppMessage>) {
        info!("Socket listener started on {}", self.socket_path.display());

        for conn in self.listener.incoming() {
            let stream = match conn {
                Ok(stream) => stream,
                Err(err) => {
                    error!(%err, "Failed to accept socket connection");
                    continue;
                }
            };

            let mut reader = BufReader::new(stream);
            let mut msg = String::new();

            if let Err(err) = reader.read_line(&mut msg) {
                error!(%err, "Error parsing socket command");
                continue;
            }

            let cmd = match msg.trim() {
                "show" => AppMessage::Show,
                "ping" => AppMessage::Ping,
                _ => {
                    error!("Invalid command {}", msg.trim());
                    continue;
                }
            };

            // Send the command to the channel
            if command_tx.send(cmd).is_err() {
                info!("Command receiver shut down");
            }
        }
    }
}

pub struct WaycastSocketClient {
    client: UnixStream,
}

// TODO: At this point I can probably just serialize app
// messages with serde instead of doing this whole
// string parsing rigamaroll.
impl WaycastSocketClient {
    pub fn new(socket_path: PathBuf) -> Result<Self, SocketError> {
        let client =
            UnixStream::connect(&socket_path).map_err(|_| SocketError::DaemonNotAvailable)?;

        Ok(Self { client })
    }

    fn send_command(&mut self, cmd: AppMessage) -> Result<(), SocketError> {
        match cmd {
            AppMessage::Show => {
                self.client.write_all(b"show\n")?;
            }
            AppMessage::Ping => {
                self.client.write_all(b"ping\n")?;
            }
            _ => panic!("Invalid command"),
        };

        Ok(())
    }

    pub fn close(&mut self) {
        let _ = self.client.shutdown(Shutdown::Write);
    }

    pub fn send_show(&mut self) -> Result<(), SocketError> {
        self.send_command(AppMessage::Show)
    }

    pub fn send_ping(&mut self) -> Result<(), SocketError> {
        self.send_command(AppMessage::Ping)
    }
}
