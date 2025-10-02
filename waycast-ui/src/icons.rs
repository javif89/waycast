use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use iced::widget::{image, svg};
use waycast_core::cache::CacheTTL;

use crate::config;

// In-memory cache for UI performance (tier 1)
static ICON_HANDLE_CACHE: OnceLock<Mutex<HashMap<String, IconHandle>>> = OnceLock::new();

#[derive(Clone)]
pub enum IconHandle {
    Svg(svg::Handle),
    Image(image::Handle),
}

pub fn get_or_load_icon(icon_name: &str) -> IconHandle {
    let cache_key = format!("{}:{}", icon_name, config::ICON_SIZE_STR);
    
    // Tier 1: Check in-memory cache first
    let cache = ICON_HANDLE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(cache_guard) = cache.lock()
        && let Some(handle) = cache_guard.get(&cache_key) {
            return handle.clone();
        }

    // Tier 2: Not in memory, get from disk cache via find_icon_file
    let icon_path = find_icon_file(icon_name, config::ICON_SIZE_STR)
        .unwrap_or_else(|| {
            find_icon_file("application-x-executable", config::ICON_SIZE_STR)
                .unwrap_or_else(|| "notfound.png".into())
        });

    // Create iced handle based on file extension
    let handle = match Path::new(&icon_path).extension().and_then(|e| e.to_str()) {
        Some("svg") => IconHandle::Svg(svg::Handle::from_path(&icon_path)),
        _ => IconHandle::Image(image::Handle::from_path(&icon_path)),
    };

    // Store in tier 1 cache for next time
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

    // Use the same caching approach as GTK UI
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