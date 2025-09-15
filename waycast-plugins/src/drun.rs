use gio::{AppInfo, DesktopAppInfo, prelude::*};
use waycast_core::{LaunchError, LauncherListItem, LauncherPlugin};
use waycast_macros::{launcher_entry, plugin};

#[derive(Debug)]
pub struct DesktopEntry {
    id: String,
    name: String,
    description: Option<String>,
    icon: String,
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
            if let Some(di) = DesktopAppInfo::new(&self.id) {
                // Get the command from the desktop entry and use our detached spawning
                if let Some(commandline) = di.commandline() {
                    let cmd_str = commandline.to_string_lossy();
                    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
                    if let Some((program, args)) = parts.split_first() {
                        crate::util::spawn_detached(program, args)
                            .map_err(|e| LaunchError::CouldNotLaunch(format!("Failed to spawn: {}", e)))?;
                        Ok(())
                    } else {
                        Err(LaunchError::CouldNotLaunch("Empty command".into()))
                    }
                } else {
                    // Fallback to GIO method for complex desktop entries
                    let app: AppInfo = di.upcast();
                    let ctx = gio::AppLaunchContext::new();
                    if app.launch(&[], Some(&ctx)).ok().is_none() {
                        return Err(LaunchError::CouldNotLaunch("App failed to launch".into()));
                    };
                    Ok(())
                }
            } else {
                Err(LaunchError::CouldNotLaunch("Invalid .desktop entry".into()))
            }
        }
    }
}

pub fn get_desktop_entries() -> Vec<DesktopEntry> {
    let mut entries = Vec::new();

    for i in gio::AppInfo::all() {
        let info: gio::DesktopAppInfo = match i.downcast_ref::<gio::DesktopAppInfo>() {
            Some(inf) => inf.to_owned(),
            None => continue,
        };
        if !info.should_show() {
            continue;
        }

        let de = DesktopEntry {
            id: info.id().unwrap_or_default().to_string(),
            name: info.display_name().to_string(),
            description: info.description().map(|d| d.to_string()),
            icon: {
                if let Some(icon) = info.icon() {
                    if let Ok(ti) = icon.clone().downcast::<gio::ThemedIcon>() {
                        // ThemedIcon may have multiple names, we take the first
                        if let Some(name) = ti.names().first() {
                            name.to_string()
                        } else {
                            "application-x-executable".to_string()
                        }
                    } else if let Ok(fi) = icon.clone().downcast::<gio::FileIcon>() {
                        if let Some(path) = fi.file().path() {
                            path.to_string_lossy().to_string()
                        } else {
                            "application-x-executable".to_string()
                        }
                    } else {
                        "application-x-executable".to_string()
                    }
                } else {
                    "application-x-executable".to_string()
                }
            },
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
