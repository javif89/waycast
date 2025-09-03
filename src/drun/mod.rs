use crate::{LaunchError, LauncherListItem};
use gio::{AppInfo, DesktopAppInfo, Icon, prelude::*};

#[derive(Debug)]
pub struct DesktopEntry {
    id: String,
    name: String,
    description: Option<glib::GString>,
    icon: Option<Icon>,
}

impl LauncherListItem for DesktopEntry {
    fn title(&self) -> String {
        return self.name.to_owned();
    }

    fn description(&self) -> Option<String> {
        if let Some(glib_string) = &self.description {
            return Some(glib_string.to_string().to_owned());
        }

        return None;
    }

    fn execute(&self) -> Result<(), LaunchError> {
        if let Some(di) = DesktopAppInfo::new(&self.id) {
            let app: AppInfo = di.upcast();
            let ctx = gio::AppLaunchContext::new();
            if app.launch(&[], Some(&ctx)).ok().is_none() {
                return Err(LaunchError::CouldNotLaunch("App failed to launch".into()));
            };
            return Ok(());
        }

        return Err(LaunchError::CouldNotLaunch("Invalid .desktop entry".into()));
    }

    fn icon(&self) -> String {
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

        return "application-x-executable".into();
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

pub fn all() -> Vec<Box<dyn LauncherListItem>> {
    let mut entries: Vec<Box<dyn LauncherListItem>> = Vec::new();

    for e in get_desktop_entries() {
        entries.push(Box::new(e));
    }

    entries
}
