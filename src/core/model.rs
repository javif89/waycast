use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum ItemKind {
    DesktopEntry,
    File,
    Project,
    Unknown,
}

impl From<String> for ItemKind {
    fn from(value: String) -> Self {
        match value.as_str() {
            "desktopentry" => Self::DesktopEntry,
            "file" => Self::File,
            "project" => Self::Project,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherItem {
    pub id: String,
    pub kind: ItemKind,
    pub title: String,
    pub description: Option<String>,
    pub icon: String,
}

pub trait WaycastScanner {
    fn scan(&self) -> Vec<LauncherItem>;
}
