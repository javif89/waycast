use std::path::PathBuf;

use thiserror::Error;

use crate::core::{
    FuzzyMatcher, ItemKind, LauncherItem,
    config::AppConfig,
    data::{DataError, WaycastData},
    icon::IconResolver,
    launcher::{self, LaunchError},
};

#[derive(Error, Debug)]
pub enum WaycastError {
    #[error(transparent)]
    Data(#[from] DataError),
    #[error(transparent)]
    Launch(#[from] LaunchError),
    #[error("Unknown item kind for {0}")]
    UnknownKind(String),
}

pub struct WaycastFacade {
    config: AppConfig,
    db: WaycastData,
    icon_resolver: IconResolver,
}

impl WaycastFacade {
    pub fn new(config: AppConfig, rt: tokio::runtime::Handle) -> Result<Self, WaycastError> {
        let db = rt.block_on(WaycastData::writeable_connection(&config.database_file))?;
        let icon_resolver = IconResolver::new(db.clone());

        Ok(Self {
            config,
            db,
            icon_resolver,
        })
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn db(&self) -> &WaycastData {
        &self.db
    }

    pub fn launch(&self, item: &LauncherItem) -> Result<(), WaycastError> {
        match item.kind {
            ItemKind::DesktopEntry => launcher::launch_desktop_entry(&item.id)?,
            ItemKind::File => launcher::open_path(&item.id)?,
            ItemKind::Project => {
                let command = self.config.project_open_command.replace("{path}", &item.id);
                launcher::run_command(&command)?
            }
            ItemKind::Unknown => return Err(WaycastError::UnknownKind(item.id.clone())),
        }

        Ok(())
    }

    /// Every distinct icon name or path referenced by an indexed item.
    pub async fn icon_names(&self) -> Result<Vec<String>, WaycastError> {
        Ok(self.db.items().get_icons().await?)
    }

    /// Resolve an icon name or path to a file on disk, falling back to the
    /// generic icon when it cannot be resolved.
    pub async fn icon_path(&self, name: &str) -> PathBuf {
        self.icon_resolver.resolve_or_fallback(name).await
    }

    /// Path to the icon used when nothing else resolves.
    pub fn fallback_icon(&self) -> PathBuf {
        self.icon_resolver.resolve_fallback()
    }

    pub async fn get_items(
        &self,
        kind: Option<ItemKind>,
    ) -> Result<Vec<LauncherItem>, WaycastError> {
        Ok(self.db.items().get_items(kind).await?)
    }

    /// Initial list of items that should be shown when no search query is
    /// present. Currently just returns desktop entries, but in the
    /// future this could account for frequently accessed items
    /// being shown and such.
    pub async fn initial_items(&self) -> Result<Vec<LauncherItem>, WaycastError> {
        self.get_items(Some(ItemKind::DesktopEntry)).await
    }

    pub async fn search(&self, query: String) -> Result<Vec<LauncherItem>, WaycastError> {
        // Use sqlite fts to filter files first since there could be thousands
        let file_results: Vec<LauncherItem> = self
            .db
            .items()
            .search(query.clone(), Some(ItemKind::File), 20)
            .await?;

        let mut fm = FuzzyMatcher::new();
        let mut rows = Vec::new();

        let apps = self.get_items(Some(ItemKind::DesktopEntry)).await?;

        let projects = self.get_items(Some(ItemKind::Project)).await?;

        rows.extend(apps);
        rows.extend(projects);

        let mut candidates = rows;
        candidates.extend(file_results);

        let results: Vec<LauncherItem> = fm
            .match_items(&query, &candidates, 5)
            .into_iter()
            .cloned()
            .collect();

        Ok(results)
    }
}
