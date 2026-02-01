use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub mod cache;

#[derive(Debug)]
pub enum LaunchError {
    CouldNotLaunch(String),
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum ItemKind {
    DesktopEntry,
    File,
    Project,
    Unknown,
}

#[derive(Debug, Clone)]
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
