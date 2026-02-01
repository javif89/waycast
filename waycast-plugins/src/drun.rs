use freedesktop::ApplicationEntry;
use waycast_core::{ItemKind, LauncherItem, WaycastScanner};

pub struct ApplicationScanner;

impl Default for ApplicationScanner {
    fn default() -> Self {
        Self
    }
}

impl WaycastScanner for ApplicationScanner {
    fn scan(&self) -> Vec<LauncherItem> {
        let apps = ApplicationEntry::all();
        let mut entries = Vec::with_capacity(apps.len());

        for app in apps {
            if !app.should_show() {
                continue;
            }

            let de = LauncherItem {
                id: app.id().unwrap_or_default().to_string(),
                kind: ItemKind::DesktopEntry,
                title: app.name().unwrap_or("Name not found".into()),
                description: app.comment().map(|d| d.to_string()),
                icon: app.icon().unwrap_or("application-x-executable".to_string()),
            };

            entries.push(de);
        }

        entries
    }
}
