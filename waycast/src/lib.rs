use crossbeam_channel::Sender;
use notify::{Config, EventKind, RecommendedWatcher};
use notify_debouncer_full::{DebouncedEvent, RecommendedCache, new_debouncer_opt, notify};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{error, info};

pub enum FileEvent {
    ChangeInDirectory,
}

/// Watch directories for file changes with a 5 second debounce.
/// Send a message to the channel when a change is detected.
pub fn watch_directories(
    directories: Vec<PathBuf>,
    comm_channel: Sender<FileEvent>,
    recursive_mode: notify::RecursiveMode,
) {
    let (tx, rx) = crossbeam_channel::unbounded();
    let notify_config = Config::default();
    let mut debouncer = new_debouncer_opt::<_, RecommendedWatcher, RecommendedCache>(
        Duration::from_secs(2),
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

                if useful_events.len() > 0 {
                    match comm_channel.send(FileEvent::ChangeInDirectory) {
                        Err(_) => error!("Failed to send directory change notification to channel"),
                        _ => {}
                    }
                }
            }
            Err(err) => println!("Watch error: {:?}", err),
        }
    }
}
