use freedesktop::ApplicationEntry;
use waycast_core::{ItemKind, LauncherItem, WaycastScanner};
pub mod projects;

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

use crossbeam_channel::unbounded;
use directories::UserDirs;
use gio::prelude::FileExt;
use glib::object::Cast;
use ignore::{WalkBuilder, WalkState};
use std::collections::HashSet;
use std::path::PathBuf;

pub struct FileScanner {
    paths: HashSet<PathBuf>,
    ignore_dirs: HashSet<String>,
}

impl Default for FileScanner {
    fn default() -> Self {
        Self {
            paths: default_search_list(),
            ignore_dirs: HashSet::new(),
        }
    }
}

impl FileScanner {
    pub fn new() -> Self {
        Self {
            paths: HashSet::new(),
            ignore_dirs: HashSet::new(),
        }
    }
    pub fn with_paths(mut self, paths: HashSet<PathBuf>) -> Self {
        self.paths.extend(paths);
        self
    }

    pub fn with_ignore_dirs(mut self, ignore_dirs: HashSet<String>) -> Self {
        self.ignore_dirs.extend(ignore_dirs);
        self
    }
}
struct Collector {
    tx: crossbeam_channel::Sender<Vec<FileEntry>>,
    local: Vec<FileEntry>,
}

impl Drop for Collector {
    fn drop(&mut self) {
        if !self.local.is_empty() {
            let _ = self.tx.send(std::mem::take(&mut self.local));
        }
    }
}

impl WaycastScanner for FileScanner {
    fn scan(&self) -> Vec<LauncherItem> {
        let mut walker = WalkBuilder::new(self.paths.iter().next().unwrap());

        for path in &self.paths {
            walker.add(path);
        }

        for dir in &self.ignore_dirs {
            walker.add_ignore(dir);
        }

        let (tx, rx) = unbounded::<Vec<FileEntry>>();
        walker
            .threads(4)
            .git_ignore(true)
            .git_exclude(true)
            .build_parallel()
            .run(|| {
                let mut collector = Collector {
                    tx: tx.clone(),
                    local: Vec::new(),
                };

                Box::new(move |result| {
                    let entry = match result {
                        Ok(e) => e,
                        Err(_) => return WalkState::Continue,
                    };

                    collector.local.push(FileEntry::from(entry));

                    WalkState::Continue
                })
            });

        // Drop the original sender so rx closes once all threads finish
        drop(tx);

        // ---- FAN-IN PHASE ----
        let mut all: Vec<FileEntry> = Vec::new();
        for mut chunk in rx.iter() {
            all.append(&mut chunk);
        }

        all.iter().map(|f| f.to_owned().into()).collect()
    }
}

#[derive(Clone)]
struct FileEntry {
    path: PathBuf,
}

impl From<ignore::DirEntry> for FileEntry {
    fn from(value: ignore::DirEntry) -> Self {
        FileEntry {
            path: value.into_path(),
        }
    }
}

impl From<walkdir::DirEntry> for FileEntry {
    fn from(value: walkdir::DirEntry) -> Self {
        FileEntry {
            path: value.into_path(),
        }
    }
}

impl From<FileEntry> for LauncherItem {
    fn from(val: FileEntry) -> Self {
        LauncherItem {
            id: val.path.to_string_lossy().to_string(),
            title: String::from(val.path.file_name().unwrap().to_string_lossy()),
            kind: waycast_core::ItemKind::File,
            description: Some(val.path.to_string_lossy().to_string()),
            icon: {
                let (content_type, _) = gio::content_type_guess(Some(&val.path), None);
                let icon = gio::content_type_get_icon(&content_type);
                if let Some(themed_icon) = icon.downcast_ref::<gio::ThemedIcon>()
                    && let Some(icon_name) = themed_icon.names().first()
                {
                    icon_name.to_string()
                } else {
                    String::from("text-x-generic")
                }
            },
        }
    }
}

pub fn default_search_list() -> HashSet<PathBuf> {
    if let Some(ud) = UserDirs::new() {
        let mut paths: HashSet<PathBuf> = HashSet::new();
        let user_dirs = [
            ud.document_dir(),
            ud.picture_dir(),
            ud.audio_dir(),
            ud.video_dir(),
        ];

        for path in user_dirs.into_iter().flatten() {
            paths.insert(path.to_path_buf());
        }

        return paths;
    }

    HashSet::new()
}
