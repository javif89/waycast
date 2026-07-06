use config::{Config, Environment};
use directories::ProjectDirs;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::sync::OnceLock;
use std::{env, path::PathBuf};
use std::{fs, io};

use crate::daemon::scanners;

/// Utility struct for waycast configuration. The idea
/// is that this resolves all needed paths and settings
/// that will then trickle down to the parts of the
/// app necessary.
#[derive(Debug)]
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
    /// Directories for app data. XDG dirs from the freedesktop spec
    pub app_dir: AppDirectories,
    /// Directories to scan for the different item types
    pub scan_paths: ScanDirectories,
}

impl AppConfig {
    pub fn development() -> Self {
        let appdirs = AppDirectories::development();
        let config_file = appdirs.config.join("waycast.toml");

        initialize(&config_file);
        // TODO: Have development specific paths when we start writing more tests
        let scan_paths = ScanDirectories::default();

        Self {
            lock_file: appdirs.runtime.join("waycast.lock"),
            socket_file: appdirs.runtime.join("waycast.sock"),
            config_file: appdirs.config.join("waycast.toml"),
            database_file: appdirs.data.join("waycast.db"),
            app_dir: appdirs,
            scan_paths,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        let appdirs = AppDirectories::default();
        let config_file = appdirs.config.join("waycast.toml");

        initialize(&config_file);
        let scan_paths = ScanDirectories::default();

        Self {
            lock_file: appdirs.runtime.join("waycast.lock"),
            socket_file: appdirs.runtime.join("waycast.sock"),
            config_file,
            database_file: appdirs.data.join("waycast.db"),
            app_dir: appdirs,
            scan_paths,
        }
    }
}

#[derive(Debug)]
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
#[derive(Debug)]
pub struct ScanDirectories {
    pub apps: HashSet<PathBuf>,
    pub projects: HashSet<PathBuf>,
    pub files: HashSet<PathBuf>,
}

impl Default for ScanDirectories {
    fn default() -> Self {
        let projects = get::<HashSet<PathBuf>>("plugins.projects.search_paths").unwrap_or_default();
        let (files, _) = get_file_search_paths();
        let apps: HashSet<PathBuf> = freedesktop::application_entry_paths().into_iter().collect();

        Self {
            apps,
            projects,
            files,
        }
    }
}

fn get_file_search_paths() -> (HashSet<PathBuf>, HashSet<String>) {
    // Gather file scanning paths and dirs to ignore
    let mut search_paths: HashSet<PathBuf> = HashSet::new();
    let mut skip_dirs: HashSet<String> = HashSet::new();

    if let Ok(paths) = config_file().get::<Vec<PathBuf>>("plugins.file_search.search_paths") {
        search_paths.extend(paths);
    } else {
        search_paths.extend(scanners::default_search_list());
    }

    if let Ok(paths) = config_file().get::<Vec<String>>("plugins.file_search.ignore_dirs") {
        skip_dirs.extend(paths);
    }

    (search_paths, skip_dirs)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WaycastConfig {}
static CONFIG_SINGLETON: OnceLock<Config> = OnceLock::new();

pub fn is_development_mode() -> bool {
    // Check if we're in development by looking for Cargo.toml in current directory
    env::current_dir()
        .map(|dir| dir.join("Cargo.toml").exists())
        .unwrap_or(false)
}

pub fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("dev.thegrind", "The Grind", "waycast")
}

pub fn config_dir() -> Option<PathBuf> {
    if let Some(dirs) = project_dirs() {
        return Some(dirs.config_dir().to_path_buf());
    }

    None
}

pub fn cache_dir() -> Option<PathBuf> {
    if is_development_mode() {
        return env::current_dir()
            .ok()
            .map(|d| d.join("xdg").join(".cache"));
    }

    if let Some(dirs) = project_dirs() {
        return Some(dirs.cache_dir().to_path_buf());
    }

    None
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

pub fn config_path<P: AsRef<Path>>(file: P) -> Option<PathBuf> {
    if let Some(p) = config_dir() {
        return Some(p.join(file));
    }

    None
}

pub fn cache_path<P: AsRef<Path>>(file: P) -> Option<PathBuf> {
    if let Some(p) = cache_dir() {
        return Some(p.join(file));
    }

    None
}

pub fn config_file() -> &'static Config {
    CONFIG_SINGLETON
        .get()
        .expect("Waycast configuration must be initialized before it is read")
}

pub fn initialize(config_path: &Path) -> &'static Config {
    CONFIG_SINGLETON.get_or_init(|| init(config_path))
}

pub fn get<T: DeserializeOwned>(key: &str) -> Result<T, config::ConfigError> {
    config_file().get::<T>(key)
}

fn init(config_path: &Path) -> Config {
    let mut cfg = Config::builder();

    cfg = cfg.add_source(config::File::with_name(&config_path.to_string_lossy()).required(false));

    cfg = cfg.add_source(Environment::with_prefix("WAYCAST"));

    cfg.build().unwrap()
}
