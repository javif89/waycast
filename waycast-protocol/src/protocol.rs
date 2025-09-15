use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn next_request_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    pub method: Method,
}

impl Request {
    pub fn new(method: Method) -> Self {
        Self {
            id: next_request_id(),
            method,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Method {
    Search(String),
    DefaultList,
    Execute(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: u64,
    pub result: std::result::Result<ResponseData, String>,
}

impl Response {
    pub fn success(id: u64, data: ResponseData) -> Self {
        Self {
            id,
            result: Ok(data),
        }
    }

    pub fn error(id: u64, message: String) -> Self {
        Self {
            id,
            result: Err(message),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseData {
    Items(Vec<LauncherItem>),
    Success,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub icon: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Connection failed: {0}")]
    Connection(#[from] std::io::Error),
    #[error("Serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Request failed: {0}")]
    Request(String),
    #[error("Daemon not running")]
    DaemonNotRunning,
    #[error("Invalid response ID")]
    InvalidResponseId,
}

pub type Result<T> = std::result::Result<T, ProtocolError>;
