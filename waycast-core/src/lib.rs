use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub mod cache;
pub mod facade;

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

pub trait LauncherPlugin: Send + Sync {
    fn init(&self) {
        // Default empty init - plugins can override this
    }
    fn name(&self) -> String;
    fn priority(&self) -> i32;
    fn description(&self) -> Option<String>;
    // Prefix to isolate results to only use this plugin
    fn prefix(&self) -> Option<String>;
    // Only search/use this plugin if the prefix was typed
    fn by_prefix_only(&self) -> bool;
    // Actual item searching functions
    fn default_list(&self) -> Vec<LauncherItem> {
        // Default empty list - plugins can override this
        Vec::new()
    }
    fn filter(&self, query: &str) -> Vec<LauncherItem>;
}
