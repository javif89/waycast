use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::Duration,
};

use directories::UserDirs;
use freedesktop::ApplicationEntry;
use gio::glib::object::Cast;
use ignore::Walk;
use tokio::time;
use tracing::{Instrument, error, info, info_span};
use tracing_subscriber::fmt;
use waycast_data::{DB, DataError, ItemRow, wal_connection};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_span_events(fmt::format::FmtSpan::CLOSE | fmt::format::FmtSpan::NEW)
        .init();

    info!("Daemon starting up...");

    let db = DB::open(wal_connection("waycast.db")).await;

    let mut cadence = time::interval(Duration::from_secs(20));

    loop {
        cadence.tick().await;

        let scan_span = info_span!("scan_and_update");

        let result = scan_and_update(&db).instrument(scan_span).await;

        match result {
            Ok(_) => info!("Items inserted successfully"),
            Err(e) => error!("Error: {e}"),
        }
    }
}

async fn scan_and_update(db: &DB) -> Result<(), DataError> {
    info!("Gathering data");
    let (desktop_entries, files) = tokio::join!(get_desktop_entries(), get_file_entries());

    info!("Found {} desktop entries", desktop_entries.len());
    info!("Found {} files", files.len());

    let mut items = Vec::with_capacity(desktop_entries.len() + files.len());
    items.extend(desktop_entries);
    items.extend(files);

    info!("Inserting {} items", items.len());
    db.insert_items(items).await?;

    Ok(())
}

async fn get_desktop_entries() -> Vec<ItemRow> {
    let mut entries = Vec::new();

    for app in ApplicationEntry::all() {
        if !app.should_show() {
            continue;
        }

        let de = ItemRow {
            id: app.id().unwrap_or_default().to_string(),
            kind: waycast_data::ItemKind::DesktopEntry,
            title: app.name().unwrap_or("Name not found".into()),
            description: app.comment().map(|d| d.to_string()),
            icon: app.icon().unwrap_or("application-x-executable".to_string()),
        };

        entries.push(de);
    }

    entries
}

pub fn default_search_list() -> HashSet<PathBuf> {
    if let Some(ud) = UserDirs::new() {
        let mut paths: HashSet<PathBuf> = HashSet::new();
        let user_dirs = [
            ud.document_dir(),
            ud.picture_dir(),
            ud.audio_dir(),
            ud.video_dir(),
        ];

        for path in user_dirs.into_iter().flatten() {
            paths.insert(path.to_path_buf());
        }

        return paths;
    }

    HashSet::new()
}

fn icon_from_path(path: impl AsRef<Path>) -> String {
    let (content_type, _) = gio::content_type_guess(Some(path.as_ref()), None);
    let icon = gio::content_type_get_icon(&content_type);
    if let Some(themed_icon) = icon.downcast_ref::<gio::ThemedIcon>()
        && let Some(icon_name) = themed_icon.names().first()
    {
        return icon_name.to_string();
    }
    String::from("text-x-generic")
}

async fn get_file_entries() -> Vec<ItemRow> {
    let mut items: Vec<ItemRow> = Vec::new();
    let paths = default_search_list();

    for p in paths {
        for result in Walk::new(p) {
            match result {
                Ok(entry) => {
                    let item = ItemRow {
                        id: entry.path().to_string_lossy().to_string(),
                        kind: waycast_data::ItemKind::File,
                        title: String::from(entry.path().file_name().unwrap().to_string_lossy()),
                        description: Some(entry.path().to_string_lossy().to_string()),
                        icon: icon_from_path(entry.path()),
                    };

                    items.push(item);
                }
                Err(_) => continue,
            }
        }
    }

    items
}
