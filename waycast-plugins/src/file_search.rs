use directories::UserDirs;
use gio::prelude::FileExt;
use glib::object::Cast;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use walkdir::{DirEntry, WalkDir};
use waycast_macros::{launcher_entry, plugin};

use crate::util::{FuzzyMatcher, FuzzySearchable, spawn_detached};
use waycast_core::{LaunchError, LauncherListItem, LauncherPlugin};

#[derive(Clone)]
struct FileEntry {
    path: PathBuf,
}

impl FileEntry {
    fn from(entry: DirEntry) -> Self {
        FileEntry {
            path: entry.into_path(),
        }
    }
}

impl LauncherListItem for FileEntry {
    launcher_entry! {
        id: self.path.to_string_lossy().to_string(),
        title: String::from(self.path.file_name().unwrap().to_string_lossy()),
        description: Some(self.path.to_string_lossy().to_string()),
        icon: {
            let (content_type, _) = gio::content_type_guess(Some(&self.path), None);
            let icon = gio::content_type_get_icon(&content_type);
            if let Some(themed_icon) = icon.downcast_ref::<gio::ThemedIcon>()
                && let Some(icon_name) = themed_icon.names().first() {
                    return icon_name.to_string();
                }
            String::from("text-x-generic")
        },
        execute: {
            println!("Executing: {}", self.path.display());

            // Use xdg-open directly since it works properly with music files
            // Detach the process so it doesn't die when daemon is killed
            match spawn_detached("xdg-open", &[self.path.to_str().unwrap()]) {
                Ok(_) => {
                    println!("Successfully launched with xdg-open");
                    Ok(())
                }
                Err(e) => {
                    println!("xdg-open failed: {}", e);
                    // Fallback to GIO method
                    let file_gio = gio::File::for_path(&self.path);
                    let ctx = gio::AppLaunchContext::new();
                    match gio::AppInfo::launch_default_for_uri(file_gio.uri().as_str(), Some(&ctx)) {
                        Ok(()) => {
                            println!("Successfully launched with GIO fallback");
                            Ok(())
                        }
                        Err(e2) => {
                            println!("GIO fallback also failed: {}", e2);
                            Err(LaunchError::CouldNotLaunch(format!(
                                "Both xdg-open and GIO failed: {} / {}",
                                e, e2
                            )))
                        }
                    }
                }
            }
        }
    }
}

impl FuzzySearchable for FileEntry {
    fn primary_key(&self) -> String {
        // Primary: filename (most common search case)
        self.path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    fn secondary_keys(&self) -> Vec<String> {
        // Secondary: full path (for directory-based searches)
        vec![self.path.to_string_lossy().to_string()]
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

pub fn default_skip_list() -> HashSet<String> {
    HashSet::from([
        String::from("vendor"),
        String::from("node_modules"),
        String::from("cache"),
        String::from("zig-cache"),
    ])
}

pub struct FileSearchPlugin {
    search_paths: HashSet<PathBuf>,
    skip_dirs: HashSet<String>,
    // Running list of files in memory
    files: Arc<Mutex<Vec<FileEntry>>>,
}

impl Default for FileSearchPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSearchPlugin {
    pub fn new() -> Self {
        let mut search_paths = default_search_list();
        let mut skip_dirs = default_skip_list();

        if let Ok(paths) =
            waycast_config::config_file().get::<Vec<PathBuf>>("plugins.file_search.search_paths")
        {
            search_paths.extend(paths);
        }

        if let Ok(paths) =
            waycast_config::config_file().get::<Vec<String>>("plugins.file_search.ignore_dirs")
        {
            skip_dirs.extend(paths);
        }

        FileSearchPlugin {
            search_paths,
            skip_dirs,
            files: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let p = path.as_ref();

        if !p.exists() {
            return Err(format!("Path does not exist: {}", p.display()));
        }

        if !p.is_dir() {
            return Err(format!("Path is not a directory: {}", p.display()));
        }

        self.search_paths.insert(p.to_path_buf());
        Ok(())
    }

    pub fn add_skip_dir(&mut self, directory_name: String) -> Result<(), String> {
        self.skip_dirs.insert(directory_name);
        Ok(())
    }

    async fn init_with_timeout(&self, timeout: Duration) {
        let files_clone = Arc::clone(&self.files);
        let skip_dirs_clone = self.skip_dirs.clone();

        println!("File search");
        println!("---Scanning directories---");
        for p in &self.search_paths {
            println!("{}", p.display());
        }

        println!("---Skipping directories---");
        for p in &self.skip_dirs {
            println!("{}", p);
        }

        let scan_task = async move {
            let mut local_files = Vec::new();
            println!("Sup");

            for path in &self.search_paths {
                let walker = WalkDir::new(path).into_iter();
                for entry in walker
                    .filter_entry(|e| {
                        !skip_hidden(e)
                            && !skip_dirs_clone.contains(e.file_name().to_string_lossy().as_ref())
                    })
                    .filter_map(|e| e.ok())
                {
                    if entry.path().is_file() {
                        local_files.push(FileEntry::from(entry));
                    }

                    // Yield control periodically to check for timeout
                    if local_files.len() % 1000 == 0 {
                        tokio::task::yield_now().await;
                    }
                }
            }

            // Update the shared files collection
            let mut files_guard = files_clone.lock().await;
            *files_guard = local_files;
        };

        // Run the scan with a timeout
        if tokio::time::timeout(timeout, scan_task).await.is_err() {
            eprintln!("File indexing timed out after {:?}", timeout);
        }
    }
}

fn skip_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

impl LauncherPlugin for FileSearchPlugin {
    plugin! {
        name: "Files",
        priority: 500,
        description: "Search and open files",
        prefix: "f"
    }

    fn init(&self) {
        // Start async file scanning with 2000ms timeout
        let self_clone = FileSearchPlugin {
            search_paths: self.search_paths.clone(),
            skip_dirs: self.skip_dirs.clone(),
            files: Arc::clone(&self.files),
        };

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                self_clone
                    .init_with_timeout(Duration::from_millis(2000))
                    .await;
            });
        });
    }

    fn filter(&self, query: &str) -> Vec<Box<dyn LauncherListItem>> {
        if query.is_empty() {
            return self.default_list();
        }

        // Try to get files without blocking - if indexing is still in progress, return empty
        let Ok(files) = self.files.try_lock() else {
            return Vec::new();
        };

        let mut fuzzy_matcher = FuzzyMatcher::new();

        // Get fuzzy matches directly on FileEntry slice
        let matches = fuzzy_matcher.match_items(query, &files, 10);

        // Convert to LauncherListItem
        matches
            .into_iter()
            .map(|file_entry| Box::new(file_entry.clone()) as Box<dyn LauncherListItem>)
            .collect()
    }
}

pub fn new() -> FileSearchPlugin {
    FileSearchPlugin::new()
}
