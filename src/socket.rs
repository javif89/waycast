use std::io::{BufRead, BufReader, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::sync::mpsc::Sender;
use std::{os::unix::net::UnixListener, path::PathBuf};

use tracing::{error, info};

#[derive(Debug)]
pub enum SocketCommand {
    /// Show the waycast UI
    Show,
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
    pub fn listen(&self, command_tx: Sender<SocketCommand>) {
        info!(
            "Socket listener started on {}",
            self.socket_path.to_string_lossy()
        );

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
                "show" => SocketCommand::Show,
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

pub struct WaycastSocketCient {
    client: UnixStream,
}

impl WaycastSocketCient {
    pub fn new(socket_path: PathBuf) -> Self {
        let client = UnixStream::connect(&socket_path).expect("Could not connect to socket");
        Self { client }
    }

    fn send_command(&mut self, cmd: SocketCommand) {
        match cmd {
            SocketCommand::Show => {
                // TODO: We should log if there's errors with the
                // socket so the user can debug
                let _ = self.client.write_all(b"show\n");
            }
        };
    }

    pub fn close(&mut self) {
        let _ = self.client.shutdown(Shutdown::Write);
    }

    pub fn send_show(&mut self) {
        self.send_command(SocketCommand::Show);
    }
}
