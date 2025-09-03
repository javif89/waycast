use crate::util::files;
use crate::{LaunchError, LauncherListItem};
use gio::{AppInfo, DesktopAppInfo, Icon, prelude::*};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct DesktopEntry {
    id: String,
    name: String,
    generic_name: Option<glib::GString>,
    description: Option<glib::GString>,
    icon: Option<Icon>,
    exec: Option<glib::GString>,
    path: PathBuf,
    no_display: bool,
    is_terminal_app: bool,
}

impl DesktopEntry {
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
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
        if let Some(di) = DesktopAppInfo::from_filename(&self.path) {
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

fn get_desktop_files() -> Vec<PathBuf> {
    let dir_envs =
        env::var("XDG_DATA_DIRS").expect("XDG_DATA_DIRS not set. Please fix your environment");
    let dir_string = String::from(dir_envs);
    let dirs = dir_string.split(":");

    let mut files = Vec::new();
    for dir in dirs {
        // println!("Data dir: {}", dir);
        let apps_path = Path::new(dir).join("applications");
        let desktop_files = match files::get_files_with_extension(&apps_path, "desktop") {
            Ok(files) => files,
            Err(_) => {
                // eprintln!("Error reading {dir}: {err}");
                continue;
            }
        };

        for f in desktop_files {
            files.push(f);
        }
    }

    return files;
}

pub fn get_desktop_entries() -> Vec<DesktopEntry> {
    let mut entries = Vec::new();

    for f in get_desktop_files() {
        if let Some(info) = DesktopAppInfo::from_filename(&f) {
            if info.is_nodisplay() {
                continue;
            }

            let de = DesktopEntry {
                id: info.id().unwrap_or_default().to_string(),
                name: info.name().to_string(),
                generic_name: info.generic_name(),
                description: info.description(),
                icon: info.icon(),
                exec: info.string("Exec"),
                no_display: info.is_nodisplay(),
                path: f.clone(),
                is_terminal_app: info.boolean("Terminal"),
            };

            entries.push(de);
        }
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
