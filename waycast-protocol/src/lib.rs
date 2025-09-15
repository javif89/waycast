pub mod protocol;
pub mod socket;
pub mod client;
pub mod server;

// Re-export commonly used types
pub use protocol::{
    Request, Response, Method, ResponseData, LauncherItem, ProtocolError, Result
};
pub use client::WaycastClient;
pub use server::{WaycastServer, RequestHandler};

// Re-export async_trait for convenience
pub use async_trait;
