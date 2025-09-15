use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

use crate::protocol::{LauncherItem, Method, Request, Response, ResponseData, Result};
use crate::socket::cleanup_socket;

#[async_trait::async_trait]
pub trait RequestHandler: Send + Sync {
    async fn search(&self, query: &str) -> std::result::Result<Vec<LauncherItem>, String>;
    async fn default_list(&self) -> std::result::Result<Vec<LauncherItem>, String>;
    async fn execute(&self, id: &str) -> std::result::Result<(), String>;
}

pub struct WaycastServer {
    listener: UnixListener,
    socket_path: std::path::PathBuf,
}

impl WaycastServer {
    pub fn new(socket_path: impl AsRef<Path>) -> Result<Self> {
        let socket_path = socket_path.as_ref().to_path_buf();

        // Clean up any existing socket
        cleanup_socket(&socket_path)?;

        let listener = UnixListener::bind(&socket_path)?;

        Ok(Self {
            listener,
            socket_path,
        })
    }

    pub async fn serve<H: RequestHandler + 'static>(self, handler: Arc<H>) -> Result<()> {
        println!("Waycast server listening on {}", self.socket_path.display());

        loop {
            match self.listener.accept().await {
                Ok((stream, _)) => {
                    let handler = Arc::clone(&handler);
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, handler).await {
                            eprintln!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }
}

impl Drop for WaycastServer {
    fn drop(&mut self) {
        let _ = cleanup_socket(&self.socket_path);
    }
}

async fn handle_connection<H: RequestHandler>(stream: UnixStream, handler: Arc<H>) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                let request: Request = match serde_json::from_str(&line) {
                    Ok(req) => req,
                    Err(e) => {
                        eprintln!("Failed to parse request: {}", e);
                        continue;
                    }
                };

                let response = handle_request(&request, handler.as_ref()).await;

                match serde_json::to_string(&response) {
                    Ok(response_json) => {
                        if let Err(e) = writer.write_all(response_json.as_bytes()).await {
                            eprintln!("Failed to write response: {}", e);
                            break;
                        }
                        if let Err(e) = writer.write_all(b"\n").await {
                            eprintln!("Failed to write newline: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to serialize response: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read from connection: {}", e);
                break;
            }
        }
    }

    Ok(())
}

async fn handle_request<H: RequestHandler>(request: &Request, handler: &H) -> Response {
    let result = match &request.method {
        Method::Search(query) => match handler.search(query).await {
            Ok(items) => Ok(ResponseData::Items(items)),
            Err(e) => Err(e),
        },
        Method::DefaultList => match handler.default_list().await {
            Ok(items) => Ok(ResponseData::Items(items)),
            Err(e) => Err(e),
        },
        Method::Execute(id) => match handler.execute(id).await {
            Ok(()) => Ok(ResponseData::Success),
            Err(e) => Err(e),
        },
    };

    Response {
        id: request.id,
        result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::*;

    struct TestHandler;

    #[async_trait::async_trait]
    impl RequestHandler for TestHandler {
        async fn search(&self, _query: &str) -> std::result::Result<Vec<LauncherItem>, String> {
            Ok(vec![LauncherItem {
                id: "test".to_string(),
                title: "Test App".to_string(),
                description: Some("Test Description".to_string()),
                icon: "test-icon".to_string(),
            }])
        }

        async fn default_list(&self) -> std::result::Result<Vec<LauncherItem>, String> {
            self.search("").await
        }

        async fn execute(&self, _id: &str) -> std::result::Result<(), String> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_handle_request() {
        let handler = TestHandler;
        let request = Request::new(Method::Search("test".to_string()));
        let response = handle_request(&request, &handler).await;

        assert_eq!(response.id, request.id);
        assert!(response.result.is_ok());
    }
}
