use notify::{Config, EventKind, RecommendedWatcher};
use notify_debouncer_full::{DebouncedEvent, RecommendedCache, new_debouncer_opt, notify};
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::{Sender, error::TrySendError};
use tracing::info;

pub enum FileEvent {
    ChangeInDirectory,
}

static DEBOUNCER_TIMEOUT_MS: u64 = 500;

/// Watch directories for file changes every DEBOUNCER_TIMEOUT_MS
/// Send a message to the channel when a change is detected.
pub fn watch_directories(
    directories: Vec<PathBuf>,
    comm_channel: Sender<FileEvent>,
    recursive_mode: notify::RecursiveMode,
) {
    let (tx, rx) = crossbeam_channel::unbounded();
    let notify_config = Config::default();
    let mut debouncer = new_debouncer_opt::<_, RecommendedWatcher, RecommendedCache>(
        Duration::from_millis(DEBOUNCER_TIMEOUT_MS),
        None,
        tx,
        RecommendedCache::new(),
        notify_config,
    )
    .expect("Could not start watcher");

    for d in directories {
        info!("Watching {}", d.to_string_lossy().to_string());
        let _ = debouncer.watch(d, recursive_mode);
    }

    for res in rx {
        match res {
            Ok(events) => {
                let useful_events: Vec<DebouncedEvent> = events
                    .into_iter()
                    .filter(|ev| {
                        matches!(
                            ev.kind,
                            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                        )
                    })
                    .collect();

                if !useful_events.is_empty() {
                    match comm_channel.try_send(FileEvent::ChangeInDirectory) {
                        Ok(()) | Err(TrySendError::Full(_)) => {}
                        Err(TrySendError::Closed(_)) => {
                            info!("Directory watcher receiver closed; stopping watcher");
                            return;
                        }
                    }
                }
            }
            Err(err) => println!("Watch error: {:?}", err),
        }
    }
}
