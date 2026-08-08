use std::{path::PathBuf, time::Duration};

use tracing::error;

use crate::core::data::{DataError, WaycastData};

/// How long a resolved icon path stays in the cache.
const CACHE_TTL: Duration = Duration::from_hours(8);

pub struct IconResolver {
    // An icon name that should always resolve. We
    // will use it when we don't provide a fallback
    // and could not find an icon.
    default_fallback_icon_path: PathBuf,
    db: WaycastData,
}

#[derive(Debug)]
pub enum IconType {
    Theme { name: String },
    Path(PathBuf),
}

impl IconResolver {
    pub fn new(db: WaycastData) -> Self {
        Self {
            default_fallback_icon_path: freedesktop::get_icon("text-x-generic").unwrap(),
            db,
        }
    }

    /// Get the path to the fallback icon.
    /// NOTE: If the fallback icon is somehow not present this
    /// will panic since it means we're in a completely
    /// messed up install. However, in the future
    /// an effort could be made to handle this
    /// more reliably.
    pub fn resolve_fallback(&self) -> PathBuf {
        self.default_fallback_icon_path.clone()
    }

    /// Resolve an icon name or path to a file on disk, reading through the
    /// cache and populating it on a miss.
    ///
    /// This doubles as the cache warmer: a caller that only wants to prime
    /// the cache can discard the returned path.
    pub async fn resolve(&self, name: &str) -> Result<Option<PathBuf>, DataError> {
        let key = self.cache_key(name);
        if let Some(cached_path) = self.db.cache().get::<PathBuf>(&key).await? {
            return Ok(Some(cached_path));
        }

        match self.resolve_uncached(name) {
            Some(resolved_path) => {
                self.db
                    .cache()
                    .put(&key, &resolved_path, Some(CACHE_TTL))
                    .await?;
                Ok(Some(resolved_path))
            }
            None => Ok(None),
        }
    }

    /// Try to resolve the icon, and return the fallback path if we can't.
    /// This should only be used in places like the UI where we HAVE to
    /// have an icon path. Can panic if the default fallback icon
    /// is not present.
    pub async fn resolve_or_fallback(&self, name: &str) -> PathBuf {
        match self.resolve(name).await {
            Ok(Some(actual)) => actual,
            Ok(None) => self.resolve_fallback(),
            Err(e) => {
                // If the database is fucked then we crash
                error!("Cache error {e}");
                panic!();
            }
        }
    }

    /// Resolve straight off the filesystem, skipping the cache entirely.
    /// NOTE: a theme lookup walks the icon directories, so this blocks.
    fn resolve_uncached(&self, name: &str) -> Option<PathBuf> {
        match self.resolve_icon_type(name) {
            IconType::Theme { name } => freedesktop::get_icon(&name),
            IconType::Path(p) => Some(p),
        }
    }

    /// If it's a named theme icon, it will resolve its path.
    /// If it's already an absolute path, it will just return
    /// back the Path variant.
    fn resolve_icon_type(&self, name: &str) -> IconType {
        // If icon_name is already a path and exists, return it directly
        let path = std::path::Path::new(name);
        // Since waycast is for linux only I'm ok with checking for /
        if path.exists() || name.contains("/") {
            return IconType::Path(path.to_path_buf());
        }

        IconType::Theme { name: name.into() }
    }

    fn cache_key(&self, icon_name: &str) -> String {
        format!("icon:{}", icon_name)
    }
}
