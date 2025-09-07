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

pub fn config_file() -> &'static Config {
    CONFIG_SINGLETON.get_or_init(|| init())
}

pub fn get<T: DeserializeOwned>(key: &str) -> Result<T, config::ConfigError> {
    config_file().get::<T>(key)
}

fn init() -> Config {
    Config::builder()
        .add_source(config::File::with_name("waycast.toml"))
        .add_source(config::Environment::with_prefix("WAYCAST"))
        .build()
        .unwrap()
}
