use config::Config;
use directories::ProjectDirs;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;
use std::{fs::File, path::PathBuf};

#[derive(Debug, Deserialize, Serialize)]
pub struct WaycastConfig {}
static CONFIG_SINGLETON: OnceLock<Config> = OnceLock::new();

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
    if let Some(dirs) = project_dirs() {
        return Some(dirs.cache_dir().to_path_buf());
    }

    None
}

pub fn data_dir() -> Option<PathBuf> {
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

pub fn config_file() -> &'static Config {
    CONFIG_SINGLETON.get_or_init(|| init())
}

pub fn get<T: DeserializeOwned>(key: &str) -> Result<T, config::ConfigError> {
    config_file().get::<T>(key)
}

fn init() -> Config {
    let mut cfg = Config::builder();

    cfg = cfg.add_source(config::File::with_name("waycast.toml").required(false)); // Local file for dev

    // Production version in ~/.config
    if let Some(path) = config_path("waycast.toml") {
        cfg = cfg.add_source(config::File::with_name(&path.to_string_lossy()).required(false));
    }

    cfg = cfg.add_source(config::Environment::with_prefix("WAYCAST"));

    cfg.build().unwrap()
}
