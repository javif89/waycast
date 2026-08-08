use directories::{ProjectDirs, UserDirs};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::{env, path::PathBuf};
use std::{fs, io};
use tracing::error;

use crate::daemon::scanners;

/// Utility struct for waycast configuration. The idea
/// is that this resolves all needed paths and settings
/// that will then trickle down to the parts of the
/// app necessary.
#[derive(Debug, Serialize)]
pub struct AppConfig {
    /// The path to the file we're using as a "single instance lock"
    /// to prevent multiple daemon/ui processes from starting.
    pub lock_file: PathBuf,
    /// The path to the unix socket we use to send commands
    /// to the daemon process.
    pub socket_file: PathBuf,
    /// Path to the waycast.toml
    pub config_file: PathBuf,
    /// Path to the waycast.db sqlite file
    pub database_file: PathBuf,
    /// Command used to open a project, with `{path}` substituted at launch.
    pub project_open_command: String,
    /// Directories for app data. XDG dirs from the freedesktop spec
    pub app_dir: AppDirectories,
    /// Directories to scan for the different item types
    pub scan_paths: ScanDirectories,
}

impl AppConfig {
    pub fn development() -> Self {
        Self::from_directories(AppDirectories::development())
    }

    fn from_directories(app_dir: AppDirectories) -> Self {
        let config_file = app_dir.config.join("waycast.toml");
        let file = WaycastConfig::load(&config_file);

        Self {
            lock_file: app_dir.runtime.join("waycast.lock"),
            socket_file: app_dir.runtime.join("waycast.sock"),
            database_file: app_dir.data.join("waycast.db"),
            scan_paths: ScanDirectories::from_file(&file),
            project_open_command: file.projects.open_command,
            config_file,
            app_dir,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::from_directories(AppDirectories::default())
    }
}

#[derive(Debug, Serialize)]
pub struct AppDirectories {
    pub config: PathBuf,
    pub cache: PathBuf,
    pub data: PathBuf,
    pub runtime: PathBuf,
}

impl AppDirectories {
    /// Development paths for all the needed files
    pub fn development() -> Self {
        let base = PathBuf::from("./xdg");

        Self {
            config: base.join(".config"),
            cache: base.join(".cache"),
            data: base.join(".data"),
            runtime: base.join(".runtime"),
        }
    }

    /// Ensure the wanted directories exist
    pub fn create(&self) -> io::Result<()> {
        let paths: Vec<&Path> = vec![&self.config, &self.cache, &self.data, &self.runtime];

        for p in paths {
            fs::create_dir_all(p)?;
        }

        Ok(())
    }
}

impl Default for AppDirectories {
    fn default() -> Self {
        let dirs = ProjectDirs::from("dev.thegrind", "The Grind", "waycast").expect("Failed to get project data directories. Please check your XDG configuration. This should not happen");

        Self {
            config: dirs.config_dir().into(),
            cache: dirs.cache_dir().into(),
            data: dirs.data_dir().into(),
            runtime: std::env::var_os("XDG_RUNTIME_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir),
        }
    }
}

/// Directories to scan for the different item types
#[derive(Debug, Serialize)]
pub struct ScanDirectories {
    pub apps: HashSet<PathBuf>,
    pub projects: HashSet<PathBuf>,
    pub files: HashSet<PathBuf>,
    /// Directory names the file scanner should skip
    pub ignore_dirs: HashSet<String>,
}

impl ScanDirectories {
    fn from_file(file: &WaycastConfig) -> Self {
        let files = if file.files.search_paths.is_empty() {
            scanners::default_search_list()
        } else {
            expand_all(&file.files.search_paths)
        };

        Self {
            apps: freedesktop::application_entry_paths().into_iter().collect(),
            projects: expand_all(&file.projects.search_paths),
            files,
            ignore_dirs: file.files.ignore_dirs.clone(),
        }
    }
}

fn expand_all(paths: &HashSet<PathBuf>) -> HashSet<PathBuf> {
    paths.iter().map(|path| expand_home(path)).collect()
}

/// `~` is a shell convention, so a path read out of the config file has to be
/// expanded by hand or it stays a literal directory name.
fn expand_home(path: &Path) -> PathBuf {
    let Ok(rest) = path.strip_prefix("~") else {
        return path.to_path_buf();
    };

    match UserDirs::new() {
        Some(dirs) => dirs.home_dir().join(rest),
        None => path.to_path_buf(),
    }
}

/// The on-disk shape of waycast.toml. See waycast.example.toml.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct WaycastConfig {
    files: FileSettings,
    projects: ProjectSettings,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileSettings {
    search_paths: HashSet<PathBuf>,
    ignore_dirs: HashSet<String>,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ProjectSettings {
    search_paths: HashSet<PathBuf>,
    open_command: String,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            search_paths: HashSet::new(),
            open_command: String::from("code -n {path}"),
        }
    }
}

impl WaycastConfig {
    /// A missing config file is normal. Anything else is reported and then
    /// falls back to defaults so the daemon still comes up.
    fn load(path: &Path) -> Self {
        let contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Self::default(),
            Err(e) => {
                error!("Could not read {}: {e}", path.display());
                return Self::default();
            }
        };

        match toml::from_str(&contents) {
            Ok(parsed) => parsed,
            Err(e) => {
                error!("Could not parse {}: {e}", path.display());
                Self::default()
            }
        }
    }
}

pub fn is_development_mode() -> bool {
    // Check if we're in development by looking for Cargo.toml in current directory
    env::current_dir()
        .map(|dir| dir.join("Cargo.toml").exists())
        .unwrap_or(false)
}

pub fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("dev.thegrind", "The Grind", "waycast")
}

pub fn data_dir() -> Option<PathBuf> {
    if is_development_mode() {
        return env::current_dir().ok().map(|d| d.join("xdg"));
    }

    if let Some(dirs) = project_dirs() {
        return Some(dirs.data_dir().to_path_buf());
    }

    None
}
