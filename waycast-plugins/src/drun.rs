use gio::{prelude::*, AppInfo, DesktopAppInfo, Icon};
use waycast_core::{LaunchError, LauncherListItem, LauncherPlugin};
use waycast_macros::{plugin, launcher_entry};

#[derive(Debug)]
pub struct DesktopEntry {
    id: String,
    name: String,
    description: Option<glib::GString>,
    icon: Option<Icon>,
}

impl LauncherListItem for DesktopEntry {
    launcher_entry! {
        id: self.id.clone(),
        title: self.name.to_owned(),
        description: {
            if let Some(glib_string) = &self.description {
                Some(glib_string.to_string().to_owned())
            } else {
                None
            }
        },
        icon: {
            if let Some(icon) = &self.icon {
                if let Ok(ti) = icon.clone().downcast::<gio::ThemedIcon>() {
                    // ThemedIcon may have multiple names, we take the first
                    if let Some(name) = ti.names().first() {
                        return name.to_string();
                    }
                }

                if let Ok(fi) = icon.clone().downcast::<gio::FileIcon>() {
                    if let Some(path) = fi.file().path() {
                        return path.to_string_lossy().to_string();
                    }
                }
            }
            "application-x-executable".into()
        },
        execute: {
            if let Some(di) = DesktopAppInfo::new(&self.id) {
                let app: AppInfo = di.upcast();
                let ctx = gio::AppLaunchContext::new();
                if app.launch(&[], Some(&ctx)).ok().is_none() {
                    return Err(LaunchError::CouldNotLaunch("App failed to launch".into()));
                };
                Ok(())
            } else {
                Err(LaunchError::CouldNotLaunch("Invalid .desktop entry".into()))
            }
        }
    }
}

pub fn get_desktop_entries() -> Vec<DesktopEntry> {
    let mut entries = Vec::new();

    for i in gio::AppInfo::all() {
        let info: gio::DesktopAppInfo;
        match i.downcast_ref::<gio::DesktopAppInfo>() {
            Some(inf) => info = inf.to_owned(),
            None => continue,
        }
        if !info.should_show() {
            continue;
        }

        let de = DesktopEntry {
            id: info.id().unwrap_or_default().to_string(),
            name: info.display_name().to_string(),
            description: info.description(),
            icon: info.icon(),
        };

        entries.push(de);
    }

    entries
}



pub struct DrunPlugin;

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
