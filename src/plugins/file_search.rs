use directories::UserDirs;
use gio::prelude::FileExt;
use glib::object::Cast;
use std::path::PathBuf;
use std::{cell::RefCell, env};
use walkdir::{DirEntry, WalkDir};

use crate::{LaunchError, LauncherListItem, LauncherPlugin};

#[derive(Clone)]
struct FileEntry {
    path: PathBuf,
}

impl FileEntry {
    fn from(entry: DirEntry) -> Self {
        return FileEntry {
            path: entry.into_path(),
        };
    }
}

impl LauncherListItem for FileEntry {
    fn title(&self) -> String {
        return String::from(self.path.file_name().unwrap().to_string_lossy());
    }
    fn description(&self) -> Option<String> {
        Some(self.path.to_string_lossy().to_string())
    }

    fn execute(&self) -> Result<(), LaunchError> {
        let file_uri = gio::File::for_path(&self.path);
        let ctx = gio::AppLaunchContext::new();
        match gio::AppInfo::launch_default_for_uri(
            file_uri.uri().as_str(),
            Some(&ctx),
        ) {
            Err(_) => Err(LaunchError::CouldNotLaunch(
                "Error opening file".to_string(),
            )),
            Ok(()) => Ok(()),
        }
    }

    fn icon(&self) -> String {
        let (content_type, _) = gio::content_type_guess(Some(&self.path), None);

        let icon = gio::content_type_get_icon(&content_type);

        if let Some(themed_icon) = icon.downcast_ref::<gio::ThemedIcon>() {
            if let Some(icon_name) = themed_icon.names().first() {
                return icon_name.to_string();
            }
        }

        String::from("text-x-generic")
    }
}

pub struct FileSearchPlugin {
    search_paths: Vec<PathBuf>,
    skip_dirs: Vec<String>,
    // Running list of files in memory
    files: RefCell<Vec<FileEntry>>,
}

impl FileSearchPlugin {
    pub fn new() -> Self {
        return FileSearchPlugin {
            search_paths: Vec::new(),
            skip_dirs: vec![
                String::from("vendor"),
                String::from("node_modules"),
                String::from("cache"),
                String::from("zig-cache"),
            ],
            files: RefCell::new(Vec::new()),
        };
    }
}

fn skip_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn skip_dir(entry: &DirEntry, dirs: &Vec<String>) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|n| dirs.contains(&String::from(n)))
        .unwrap_or(false)
}

impl LauncherPlugin for FileSearchPlugin {
    fn init(&self) {
        // let home = env::home_dir().unwrap();
        if let Some(ud) = UserDirs::new() {
            let scan = [
                ud.document_dir(),
                ud.picture_dir(),
                ud.audio_dir(),
                ud.video_dir(),
            ];

            for p in scan {
                match p {
                    Some(path) => {
                        let walker = WalkDir::new(path).into_iter();
                        for entry in walker
                            .filter_entry(|e| !skip_hidden(e) && !skip_dir(e, &self.skip_dirs))
                            .filter_map(|e| e.ok())
                        {
                            if entry.path().is_file() {
                                self.files.borrow_mut().push(FileEntry::from(entry));
                            }
                        }
                    }
                    None => continue,
                }
            }
        }
    }
    fn name(&self) -> String {
        return String::from("File search");
    }

    fn priority(&self) -> i32 {
        return 900;
    }

    fn description(&self) -> Option<String> {
        None
    }

    fn prefix(&self) -> Option<String> {
        Some(String::from("f"))
    }

    fn by_prefix_only(&self) -> bool {
        false
    }

    fn default_list(&self) -> Vec<Box<dyn LauncherListItem>> {
        Vec::new()
    }

    fn filter(&self, query: &str) -> Vec<Box<dyn LauncherListItem>> {
        if query.is_empty() {
            return self.default_list();
        }

        let mut entries: Vec<Box<dyn LauncherListItem>> = Vec::new();
        let files = self.files.borrow();
        for f in files.iter() {
            if let Some(file_name) = f.path.file_name() {
                let cmp = file_name.to_string_lossy().to_lowercase();
                if cmp.contains(&query.to_lowercase()) {
                    entries.push(Box::new(f.clone()));
                }
            }
        }

        entries
    }
}
