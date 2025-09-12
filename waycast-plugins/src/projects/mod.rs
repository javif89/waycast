pub mod framework_detector;
pub mod framework_macro;
pub mod type_scanner;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use std::sync::LazyLock;
use tokio::sync::Mutex;
use waycast_core::{
    cache::{Cache, CacheTTL},
    LaunchError, LauncherListItem, LauncherPlugin,
};
use waycast_macros::{launcher_entry, plugin};

use crate::projects::{framework_detector::FrameworkDetector, type_scanner::TypeScanner};

static TOKEI_SCANNER: LazyLock<TypeScanner> = LazyLock::new(TypeScanner::new);
static FRAMEWORK_DETECTOR: LazyLock<FrameworkDetector> = LazyLock::new(FrameworkDetector::new);

#[derive(Clone)]
pub struct ProjectEntry {
    path: PathBuf,
    exec_command: Arc<str>,
    project_type: Option<String>,
}

impl LauncherListItem for ProjectEntry {
    launcher_entry! {
        id: self.path.to_string_lossy().to_string(),
        title: String::from(self.path.file_name().unwrap().to_string_lossy()),
        description: Some(self.path.to_string_lossy().to_string()),
        icon: {
            if let Some(t) = &self.project_type {
                // Try XDG data directory first, fall back to development path
                let icon_name = format!("{}.svg", t.to_lowercase());

                if let Some(data_dir) = waycast_config::data_dir() {
                    let xdg_icon_path = data_dir.join("icons").join("devicons").join(&icon_name);
                    if xdg_icon_path.exists() {
                        return xdg_icon_path.to_string_lossy().to_string();
                    }
                }

                // Fall back to development path
                let dev_icon_path = PathBuf::from("./assets/icons/devicons").join(&icon_name);
                if dev_icon_path.exists() {
                    return dev_icon_path.to_string_lossy().to_string();
                }
            }

            String::from("vscode")
        },
        execute: {
            let project_path = self.path.to_string_lossy().to_string();
            let exec_cmd = self.exec_command.replace("{path}", &project_path);
            let parts: Vec<&str> = exec_cmd.split_whitespace().collect();
            if let Some((program, args)) = parts.split_first() {
                let mut cmd = Command::new(program);
                cmd.args(args);
                match cmd.spawn() {
                    Ok(_) => {
                        println!("Successfully opened with code");
                        Ok(())
                    }
                    Err(_) => Err(LaunchError::CouldNotLaunch("Failed to open project folder".into())),
                }
            } else {
                Err(LaunchError::CouldNotLaunch("No program found in exec_command".into()))
            }
        }
    }
}

pub struct ProjectsPlugin {
    search_paths: HashSet<PathBuf>,
    skip_dirs: HashSet<String>,
    open_command: String,
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

        self.search_paths.insert(p.to_path_buf());
        Ok(())
    }

    pub fn add_skip_dir(&mut self, directory_name: String) -> Result<(), String> {
        self.skip_dirs.insert(directory_name);
        Ok(())
    }
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
        let exec_command = Arc::from(self.open_command.as_str());

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
                                            if skip_dirs.contains(name_str) {
                                                continue;
                                            }

                                            let project_type = detect_project_type(
                                                path.to_string_lossy().to_string().as_str(),
                                            );
                                            project_entries.push(ProjectEntry {
                                                path,
                                                exec_command: Arc::clone(&exec_command),
                                                project_type,
                                            });
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
    let search_paths =
        match waycast_config::get::<HashSet<PathBuf>>("plugins.projects.search_paths") {
            Ok(paths) => paths,
            Err(_) => HashSet::new(),
        };

    let skip_dirs = match waycast_config::get::<HashSet<String>>("plugins.projects.skip_dirs") {
        Ok(paths) => paths,
        Err(_) => HashSet::new(),
    };

    let open_command = match waycast_config::get::<String>("plugins.projects.open_command") {
        Ok(cmd) => cmd,
        Err(_) => String::from("code -n {path}"),
    };

    ProjectsPlugin {
        search_paths,
        skip_dirs,
        open_command,
        files: Arc::new(Mutex::new(Vec::new())),
    }
}

fn detect_project_type(path: &str) -> Option<String> {
    let cache_key = format!("project_type:{}", path);
    let cache = waycast_core::cache::get();

    let detect_fn = |path| {
        let fw = FRAMEWORK_DETECTOR.detect(path);
        if let Some(name) = fw {
            return Some(name);
        } else {
            let langs = TOKEI_SCANNER.scan(path, Some(1));
            if let Some(l) = langs.first() {
                // We do some special replacements so it's easier to match
                // with the icon file names
                return Some(l.name.to_owned().replace("+", "plus").replace("#", "sharp"));
            }
        }

        None
    };

    let result: Result<Option<String>, waycast_core::cache::errors::CacheError> =
        cache.remember_with_ttl(&cache_key, CacheTTL::hours(24), || detect_fn(path));

    if let Ok(project_type) = result {
        return project_type;
    }

    detect_fn(path)
}
