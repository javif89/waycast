use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use crate::protocol::{
    LauncherItem, Method, ProtocolError, Request, Response, ResponseData, Result,
};
use crate::socket::default_socket_path;

pub struct WaycastClient {
    stream: UnixStream,
}

impl WaycastClient {
    pub fn connect() -> Result<Self> {
        let socket_path = default_socket_path()?;
        Self::connect_to(&socket_path)
    }

    pub fn connect_to(socket_path: impl AsRef<std::path::Path>) -> Result<Self> {
        let stream =
            UnixStream::connect(socket_path).map_err(|_| ProtocolError::DaemonNotRunning)?;

        Ok(Self { stream })
    }

    fn send_request(&mut self, request: Request) -> Result<Response> {
        // Send the request
        let request_json = serde_json::to_string(&request)?;
        self.stream.write_all(request_json.as_bytes())?;
        self.stream.write_all(b"\n")?;
        self.stream.flush()?;

        // Read the response
        let mut reader = BufReader::new(&self.stream);
        let mut line = String::new();
        reader.read_line(&mut line)?;

        let response: Response = serde_json::from_str(&line)?;
        Ok(response)
    }

    pub fn search(&mut self, query: &str) -> Result<Vec<LauncherItem>> {
        let request = Request::new(Method::Search(query.to_string()));
        let response = self.send_request(request)?;

        match response.result {
            Ok(ResponseData::Items(items)) => Ok(items),
            Ok(_) => Err(ProtocolError::Request(
                "Expected items response".to_string(),
            )),
            Err(e) => Err(ProtocolError::Request(e)),
        }
    }

    pub fn default_list(&mut self) -> Result<Vec<LauncherItem>> {
        let request = Request::new(Method::DefaultList);
        let response = self.send_request(request)?;

        match response.result {
            Ok(ResponseData::Items(items)) => Ok(items),
            Ok(_) => Err(ProtocolError::Request(
                "Expected items response".to_string(),
            )),
            Err(e) => Err(ProtocolError::Request(e)),
        }
    }

    pub fn execute(&mut self, id: &str) -> Result<()> {
        let request = Request::new(Method::Execute(id.to_string()));
        let response = self.send_request(request)?;

        match response.result {
            Ok(ResponseData::Success) => Ok(()),
            Ok(_) => Err(ProtocolError::Request(
                "Expected success response".to_string(),
            )),
            Err(e) => Err(ProtocolError::Request(e)),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::protocol::*;

    #[test]
    fn test_request_creation() {
        let request = Request::new(Method::Search("test".to_string()));
        assert!(request.id > 0);
        assert!(matches!(request.method, Method::Search(_)));
    }
}
