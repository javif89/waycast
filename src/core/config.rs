use config::{Config, Environment};
use directories::ProjectDirs;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;
use std::{env, path::PathBuf};
use std::{fs, io};

/// Utility struct for waycast configuration. The idea
/// is that this resolves all needed paths and settings
/// that will then trickle down to the parts of the
/// app necessary. Will be loosely integrated for now.
/// TODO: Add in project and file scan directories from waycast.toml
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
}

impl AppConfig {
    pub fn development() -> Self {
        let appdirs = AppDirectories::development();

        Self {
            lock_file: appdirs.runtime.join("waycast.lock"),
            socket_file: appdirs.runtime.join("waycast.sock"),
            config_file: appdirs.config.join("waycast.toml"),
            database_file: appdirs.data.join("waycast.db"),
            app_dir: appdirs,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        let appdirs = AppDirectories::default();

        Self {
            lock_file: appdirs.runtime.join("waycast.lock"),
            socket_file: appdirs.runtime.join("waycast.sock"),
            config_file: appdirs.config.join("waycast.toml"),
            database_file: appdirs.data.join("waycast.db"),
            app_dir: appdirs,
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
