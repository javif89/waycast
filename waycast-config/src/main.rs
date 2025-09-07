use std::collections::HashMap;

use config::Config;
use directories::ProjectDirs;
pub fn main() {
    // if let Some(dirs) = ProjectDirs::from("dev.thegrind", "The Grind", "waycast") {
    //     let config_dir = dirs.config_dir();
    //     let cache_dir = dirs.cache_dir();
    //     let data_dir = dirs.data_dir();

    //     println!("Config: {}", config_dir.display());
    //     println!("Cache: {}", cache_dir.display());
    //     println!("Data: {}", data_dir.display());
    // }

    // let settings = Config::builder()
    //     .add_source(config::File::with_name("waycast.toml"))
    //     .add_source(config::Environment::with_prefix("WAYCAST"))
    //     .build()
    //     .unwrap();

    println!(
        "{:?}",
        waycast_config::config_file()
            .get::<String>("plugins.projects.open_command")
            .expect("Could not deserialize")
    );

    // // Start with defaults
    // let mut config = WaycastConfig::default();

    // config.plugins.file_search.ignore_dirs =
    //     vec!["vendor".into(), "pycache".into(), "node_modules".into()];

    // // Serialize to TOML
    // let toml_str = toml::to_string_pretty(&config).expect("Failed");
    // println!("Serialized TOML:\n{}", toml_str);

    // // Deserialize back
    // let parsed: WaycastConfig = toml::from_str(&toml_str).expect("Fuck");
    // println!("Deserialized struct:\n{:#?}", parsed);
}
