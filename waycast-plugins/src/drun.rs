use std::path::PathBuf;

use freedesktop::ApplicationEntry;
use gio::{AppInfo, DesktopAppInfo, prelude::*};
use waycast_core::{LaunchError, LauncherListItem, LauncherPlugin};
use waycast_macros::{launcher_entry, plugin};

#[derive(Debug)]
pub struct DesktopEntry {
    id: String,
    name: String,
    description: Option<String>,
    icon: String,
    path: PathBuf,
}

impl LauncherListItem for DesktopEntry {
    launcher_entry! {
        id: self.id.clone(),
        title: self.name.to_owned(),
        description: {
            self.description.as_ref().map(|glib_string| glib_string.to_string().to_owned())
        },
        icon: {
            self.icon.to_owned()
        },
        execute: {
            let app = ApplicationEntry::from_path(&self.path);

            match app.execute() {
                Ok(_) => Ok(()),
                Err(_) => Err(LaunchError::CouldNotLaunch("Failed to launch app".into()))
            }
        }
    }
}

pub fn get_desktop_entries() -> Vec<DesktopEntry> {
    let mut entries = Vec::new();

    for app in ApplicationEntry::all() {
        if !app.should_show() {
            continue;
        }

        let de = DesktopEntry {
            id: app.id().unwrap_or_default().to_string(),
            name: app.name().unwrap_or("Name not found".into()),
            description: app.comment().map(|d| d.to_string()),
            icon: app.icon().unwrap_or("application-x-executable".to_string()),
            path: app.path().into(),
        };

        entries.push(de);
    }

    entries
}

pub struct DrunPlugin;

impl Default for DrunPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl DrunPlugin {
    pub fn new() -> Self {
        DrunPlugin
    }
}

impl LauncherPlugin for DrunPlugin {
    plugin! {
        name: "drun",
        priority: 1000,
        description: "List and launch an installed application",
        prefix: "app"
    }

    fn default_list(&self) -> Vec<Box<dyn LauncherListItem>> {
        let mut entries: Vec<Box<dyn LauncherListItem>> = Vec::new();

        for e in get_desktop_entries() {
            entries.push(Box::new(e));
        }

        entries
    }

    fn filter(&self, query: &str) -> Vec<Box<dyn LauncherListItem>> {
        if query.is_empty() {
            return self.default_list();
        }

        let query_lower = query.to_lowercase();
        let mut entries: Vec<Box<dyn LauncherListItem>> = Vec::new();
        for entry in self.default_list() {
            let title_lower = entry.title().to_lowercase();
            if title_lower.contains(&query_lower) {
                entries.push(entry);
            }
        }

        entries
    }
}

pub fn new() -> DrunPlugin {
    DrunPlugin::new()
}
