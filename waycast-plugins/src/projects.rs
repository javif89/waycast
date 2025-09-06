// TODO: Use the user's preferred editor.
// This should just be in the config when I implement
// that eventually since figuring out every editor's
// launch option would be a pain. The user can just
// configure launch_command and pass a parameter.
// Example: code -n {path}
// and I'll just regex in the path.
// TODO: Project type detection and icon
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use tokio::sync::Mutex;
use waycast_core::{LaunchError, LauncherListItem, LauncherPlugin};
use waycast_macros::{launcher_entry, plugin};

#[derive(Clone)]
pub struct ProjectEntry {
    path: PathBuf,
}

impl LauncherListItem for ProjectEntry {
    launcher_entry! {
        id: self.path.to_string_lossy().to_string(),
        title: String::from(self.path.file_name().unwrap().to_string_lossy()),
        description: Some(self.path.to_string_lossy().to_string()),
        icon: {
            String::from("vscode")
        },
        execute: {
            // Use xdg-open directly since it works properly with music files
            match Command::new("code").arg("-n").arg(&self.path).spawn() {
                Ok(_) => {
                    println!("Successfully opened with code");
                    Ok(())
                }
                Err(_) => Err(LaunchError::CouldNotLaunch("Failed to open project folder".into())),
            }
        }
    }
}

pub struct ProjectsPlugin {
    search_paths: Vec<PathBuf>,
    skip_dirs: Vec<String>,
    // Running list of files in memory
    files: Arc<Mutex<Vec<ProjectEntry>>>,
}

impl ProjectsPlugin {
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
}

fn should_skip_dir(dir_name: &str, skip_dirs: &[String]) -> bool {
    skip_dirs.iter().any(|skip| skip == dir_name)
}

impl LauncherPlugin for ProjectsPlugin {
    plugin! {
        name: "Projects",
        priority: 800,
        description: "Search and open code projects",
        prefix: "proj"
    }

    fn init(&self) {
        let files_clone = Arc::clone(&self.files);
        let search_paths = self.search_paths.clone();
        let skip_dirs = self.skip_dirs.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut project_entries = Vec::new();

                for search_path in &search_paths {
                    if let Ok(entries) = fs::read_dir(search_path) {
                        for entry in entries.flatten() {
                            if let Ok(file_type) = entry.file_type() {
                                if file_type.is_dir() {
                                    let path = entry.path();

                                    // Skip hidden directories (starting with .)
                                    if let Some(file_name) = path.file_name() {
                                        if let Some(name_str) = file_name.to_str() {
                                            // Skip hidden directories
                                            if name_str.starts_with('.') {
                                                continue;
                                            }

                                            // Skip directories in skip list
                                            if should_skip_dir(name_str, &skip_dirs) {
                                                continue;
                                            }

                                            project_entries.push(ProjectEntry { path });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Update the shared files collection
                let mut files_guard = files_clone.lock().await;
                *files_guard = project_entries;

                println!("Projects plugin: Found {} projects", files_guard.len());
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

pub fn new() -> ProjectsPlugin {
    ProjectsPlugin {
        search_paths: Vec::new(),
        skip_dirs: vec![
            String::from("vendor"),
            String::from("node_modules"),
            String::from("cache"),
            String::from("zig-cache"),
            String::from(".git"),
            String::from(".svn"),
        ],
        files: Arc::new(Mutex::new(Vec::new())),
    }
}
