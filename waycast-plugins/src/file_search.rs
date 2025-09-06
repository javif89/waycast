use directories::UserDirs;
use gio::prelude::FileExt;
use glib::object::Cast;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use walkdir::{DirEntry, WalkDir};
use waycast_macros::{plugin, launcher_entry};

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
            if let Some(themed_icon) = icon.downcast_ref::<gio::ThemedIcon>() {
                if let Some(icon_name) = themed_icon.names().first() {
                    return icon_name.to_string();
                }
            }
            String::from("text-x-generic")
        },
        execute: {
            println!("Executing: {}", self.path.display());

            // Use xdg-open directly since it works properly with music files
            match Command::new("xdg-open").arg(&self.path).spawn() {
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

pub fn default_search_list() -> Vec<PathBuf> {
    if let Some(ud) = UserDirs::new() {
        let mut paths: Vec<PathBuf> = Vec::new();
        let user_dirs = [
            ud.document_dir(),
            ud.picture_dir(),
            ud.audio_dir(),
            ud.video_dir(),
        ];

        for path in user_dirs.into_iter().flatten() {
            paths.push(path.to_path_buf());
        }

        return paths;
    }

    Vec::new()
}

pub struct FileSearchPlugin {
    search_paths: Vec<PathBuf>,
    skip_dirs: Vec<String>,
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
        FileSearchPlugin {
            search_paths: default_search_list(),
            skip_dirs: vec![
                String::from("vendor"),
                String::from("node_modules"),
                String::from("cache"),
                String::from("zig-cache"),
            ],
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

        self.search_paths.push(p.to_path_buf());
        Ok(())
    }

    pub fn add_skip_dir(&mut self, directory_name: String) -> Result<(), String> {
        self.skip_dirs.push(directory_name);
        Ok(())
    }

    async fn init_with_timeout(&self, timeout: Duration) {
        let files_clone = Arc::clone(&self.files);
        let skip_dirs_clone = self.skip_dirs.clone();

        let scan_task = async move {
            let mut local_files = Vec::new();

            for path in &self.search_paths {
                let walker = WalkDir::new(path).into_iter();
                for entry in walker
                    .filter_entry(|e| !skip_hidden(e) && !skip_dir(e, &skip_dirs_clone))
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
        if let Err(_) = tokio::time::timeout(timeout, scan_task).await {
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

fn skip_dir(entry: &DirEntry, dirs: &Vec<String>) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|n| dirs.contains(&String::from(n)))
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

        let mut entries: Vec<Box<dyn LauncherListItem>> = Vec::new();

        // Try to get files without blocking - if indexing is still in progress, return empty
        if let Ok(files) = self.files.try_lock() {
            for f in files.iter() {
                if let Some(file_name) = f.path.file_name() {
                    let cmp = file_name.to_string_lossy().to_lowercase();
                    if cmp.contains(&query.to_lowercase()) {
                        entries.push(Box::new(f.clone()));
                    }
                }
            }
        }

        entries
    }
}




pub fn new() -> FileSearchPlugin {
    FileSearchPlugin::new()
}