use serde::{Deserialize, Serialize};
use waycast_core::LauncherListItem;
use zbus::zvariant::Type;
use zbus::interface;
pub mod client;
use std::sync::{Arc, Mutex};
use waycast_core::WaycastLauncher;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LauncherItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub icon: String,
}

impl From<&Box<dyn LauncherListItem>> for LauncherItem {
    fn from(value: &Box<dyn LauncherListItem>) -> Self {
        LauncherItem {
            id: value.id(),
            title: value.title(),
            description: value.description(),
            icon: value.icon(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Response {
    pub response_type: ResponseType,
    pub items: Option<Vec<LauncherItem>>,
    pub error: Option<String>,
}

impl Response {
    pub fn success() -> Self {
        Response {
            response_type: ResponseType::Success,
            items: None,
            error: None,
        }
    }

    pub fn error<S: Into<String>>(message: S) -> Self {
        Response {
            response_type: ResponseType::Error,
            items: None,
            error: Some(message.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum ResponseType {
    Items,
    Success,
    Error,
}

impl From<&Vec<Box<dyn LauncherListItem>>> for Response {
    fn from(value: &Vec<Box<dyn LauncherListItem>>) -> Self {
        let items = value.iter().map(|r| LauncherItem::from(r)).collect();

        Response {
            response_type: ResponseType::Items,
            items: Some(items),
            error: None,
        }
    }
}

pub struct WaycastService {
    launcher: Arc<Mutex<WaycastLauncher>>,
}

impl WaycastService {
    pub fn new(launcher: WaycastLauncher) -> Self {
        Self {
            launcher: Arc::new(Mutex::new(launcher)),
        }
    }
}

#[interface(
    name = "dev.waycast.Daemon",
    proxy(
        default_path = "/dev/waycast/Daemon",
        default_service = "dev.waycast.Daemon",
    )
)]
impl WaycastService {
    fn search(&self, query: &str) -> Response {
        let mut launcher = self.launcher.lock().unwrap();
        launcher.search(query);

        Response::from(launcher.current_results())
    }

    fn default_list(&self) -> Response {
        let mut launcher = self.launcher.lock().unwrap();
        launcher.get_default_results();

        Response::from(launcher.current_results())
    }

    fn execute(&self, id: &str) -> Response {
        let launcher = self.launcher.lock().unwrap();

        if let Err(_) = launcher.execute_item_by_id(id) {
            return Response::error(format!("Failed to launch item: {}", id));
        }

        Response::success()
    }
}
