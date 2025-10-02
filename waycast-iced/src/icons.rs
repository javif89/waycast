use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use iced::widget::{image, svg};
use waycast_core::cache::CacheTTL;

use crate::config;

static ICON_CACHE: OnceLock<Mutex<HashMap<String, IconHandle>>> = OnceLock::new();

#[derive(Clone)]
pub enum IconHandle {
    Svg(svg::Handle),
    Image(image::Handle),
}

pub fn init_cache() {
    ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
}

pub fn get_or_load_icon(icon_name: &str) -> IconHandle {
    let cache = ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let cache_key = format!("icon:{}", icon_name);

    // Check if already cached
    if let Ok(cache_guard) = cache.lock() {
        if let Some(handle) = cache_guard.get(&cache_key) {
            return handle.clone();
        }
    }

    // Load the icon
    let icon_path = find_icon_file(icon_name, config::ICON_SIZE_STR)
        .unwrap_or_else(|| {
            find_icon_file("application-x-executable", config::ICON_SIZE_STR)
                .unwrap_or_else(|| "notfound.png".into())
        });

    let handle = match Path::new(&icon_path).extension().and_then(|e| e.to_str()) {
        Some("svg") => IconHandle::Svg(svg::Handle::from_path(&icon_path)),
        _ => IconHandle::Image(image::Handle::from_path(&icon_path)),
    };

    // Store in cache
    if let Ok(mut cache_guard) = cache.lock() {
        cache_guard.insert(cache_key, handle.clone());
    }

    handle
}

fn find_icon_file(icon_name: &str, size: &str) -> Option<std::path::PathBuf> {
    // If icon_name is already a path and exists, return it directly
    let path = std::path::Path::new(icon_name);
    if path.exists() {
        return Some(path.to_path_buf());
    }

    let cache_key = format!("icon:{}:{}", icon_name, size);
    let cache = waycast_core::cache::get();

    let result = cache.remember_with_ttl(&cache_key, CacheTTL::hours(24), || {
        freedesktop::get_icon(icon_name)
    });

    if let Ok(opt_path) = result {
        return opt_path;
    }

    freedesktop::get_icon(icon_name)
}