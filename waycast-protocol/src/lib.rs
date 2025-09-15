pub mod client;
pub mod protocol;
pub mod server;
pub mod socket;

// Re-export commonly used types
pub use client::WaycastClient;
pub use protocol::{LauncherItem, Method, ProtocolError, Request, Response, ResponseData, Result};
pub use server::{RequestHandler, WaycastServer};

// Re-export async_trait for convenience
pub use async_trait;
