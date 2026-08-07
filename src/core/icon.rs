use std::{path::PathBuf, sync::Arc, time::Duration};

use tracing::error;

use crate::core::data::{DataError, WaycastData};

pub struct IconResolver {
    // An icon name that should always resolve. We
    // will use it when we don't provide a fallback
    // and could not find an icon.
    default_fallback_icon_path: PathBuf,
}

#[derive(Debug)]
pub enum IconType {
    Theme { name: String },
    Path(PathBuf),
}

// TODO
// - Add configurability for defaults if not found
impl Default for IconResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl IconResolver {
    pub fn new() -> Self {
        Self {
            default_fallback_icon_path: freedesktop::get_icon("text-x-generic").unwrap(),
        }
    }

    /// If it's a named theme icon, it will resolve its path.
    /// If it's already an absolute path, it will just return
    /// back the Path variant.
    pub fn resolve_icon_type(&self, name: &str) -> IconType {
        // If icon_name is already a path and exists, return it directly
        let path = std::path::Path::new(name);
        // Since waycast is for linux only I'm ok with checking for /
        if path.exists() || name.contains("/") {
            return IconType::Path(path.to_path_buf());
        }

        IconType::Theme { name: name.into() }
    }

    pub fn resolve(&self, name: &str) -> Option<PathBuf> {
        match self.resolve_icon_type(name) {
            IconType::Theme { name } => freedesktop::get_icon(&name),
            IconType::Path(p) => Some(p),
        }
    }

    pub fn cache_key(&self, icon_name: &str) -> String {
        format!("icon:{}", icon_name)
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

    // TODO: When I have an actual application container, this should either
    // 1. Not take DB at all and use self.db OR
    // 2. Make the resolve() method do this functionality as well
    /// Get icon from the cache or try to resolve on miss
    pub async fn resolve_cached(
        &self,
        db: Arc<WaycastData>,
        name: &str,
    ) -> Result<Option<PathBuf>, DataError> {
        let key = self.cache_key(name);
        if let Some(cached_path) = db.cache().get::<PathBuf>(&key).await? {
            return Ok(Some(cached_path));
        }

        match self.resolve(name) {
            Some(resolved_path) => {
                db.cache()
                    .put(&key, &resolved_path, Some(Duration::from_hours(8)))
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
    pub async fn resolve_or_fallback_cached(&self, db: Arc<WaycastData>, name: &str) -> PathBuf {
        match self.resolve_cached(db.clone(), name).await {
            Ok(opt_path) => match opt_path {
                Some(actual) => actual,
                None => self.resolve_fallback(),
            },
            Err(e) => {
                // If the database is fucked then we crash
                error!("Cache error {e}");
                panic!();
            }
        }
    }
}
