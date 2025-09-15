use serde::{Deserialize, Serialize};
use waycast_core::LauncherListItem;
use zvariant::Type;

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
    response_type: ResponseType,
    items: Option<Vec<LauncherItem>>,
    error: Option<String>,
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
        let items = value.iter().map(|r| Some(LauncherItem::from(r))).collect();

        Response {
            response_type: ResponseType::Items,
            items,
            error: None,
        }
    }
}
