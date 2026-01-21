use std::path::PathBuf;

use freedesktop::ApplicationEntry;
use waycast_core::{ItemKind, LauncherItem, LauncherPlugin};
use waycast_macros::plugin;

use crate::util::{FuzzyMatcher, FuzzySearchable};

#[derive(Debug, Clone)]
pub struct DesktopEntry {
    id: String,
    name: String,
    description: Option<String>,
    icon: String,
    path: PathBuf,
}

impl Into<LauncherItem> for DesktopEntry {
    fn into(self) -> LauncherItem {
        LauncherItem {
            id: self.id.clone(),
            kind: ItemKind::DesktopEntry,
            title: self.name.to_owned(),
            description: {
                self.description
                    .as_ref()
                    .map(|glib_string| glib_string.to_string().to_owned())
            },
            icon: self.icon.to_owned(),
        }
    }
}

impl FuzzySearchable for DesktopEntry {
    fn primary_key(&self) -> String {
        self.name.to_owned()
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

    fn default_list(&self) -> Vec<LauncherItem> {
        let mut entries: Vec<LauncherItem> = Vec::new();

        for e in get_desktop_entries() {
            entries.push(e.into());
        }

        entries
    }

    fn filter(&self, query: &str) -> Vec<LauncherItem> {
        if query.is_empty() {
            return self.default_list();
        }

        let mut fm = FuzzyMatcher::new();
        let entries = get_desktop_entries();
        let matches = fm.match_items(query, &entries, 5);

        matches.into_iter().map(|de| de.to_owned().into()).collect()
    }
}

pub fn new() -> DrunPlugin {
    DrunPlugin::new()
}
