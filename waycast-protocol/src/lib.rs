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
