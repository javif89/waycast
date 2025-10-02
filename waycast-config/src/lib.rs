use config::{Config, Environment};
use directories::ProjectDirs;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;
use std::{env, path::PathBuf};

#[derive(Debug, Deserialize, Serialize)]
pub struct WaycastConfig {}
static CONFIG_SINGLETON: OnceLock<Config> = OnceLock::new();

fn is_development_mode() -> bool {
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
        return env::current_dir()
            .ok()
            .map(|d| d.join("xdg").join(".local/share"));
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
    CONFIG_SINGLETON.get_or_init(init)
}

pub fn get<T: DeserializeOwned>(key: &str) -> Result<T, config::ConfigError> {
    config_file().get::<T>(key)
}

fn init() -> Config {
    let mut cfg = Config::builder();

    if let Some(path) = config_path("waycast.toml") {
        cfg = cfg.add_source(config::File::with_name(&path.to_string_lossy()).required(false));
    }

    cfg = cfg.add_source(Environment::with_prefix("WAYCAST"));

    cfg.build().unwrap()
}
