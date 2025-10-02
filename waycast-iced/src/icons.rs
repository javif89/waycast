use std::path::Path;

use iced::widget::{image, svg};
use waycast_core::cache::CacheTTL;

use crate::config;

#[derive(Clone)]
pub enum IconHandle {
    Svg(svg::Handle),
    Image(image::Handle),
}

pub fn get_or_load_icon(icon_name: &str) -> IconHandle {
    // Use the same icon finding logic as GTK UI, which uses waycast_core cache
    let icon_path = find_icon_file(icon_name, config::ICON_SIZE_STR)
        .unwrap_or_else(|| {
            find_icon_file("application-x-executable", config::ICON_SIZE_STR)
                .unwrap_or_else(|| "notfound.png".into())
        });

    // Create iced handle based on file extension
    match Path::new(&icon_path).extension().and_then(|e| e.to_str()) {
        Some("svg") => IconHandle::Svg(svg::Handle::from_path(&icon_path)),
        _ => IconHandle::Image(image::Handle::from_path(&icon_path)),
    }
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