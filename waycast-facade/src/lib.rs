use freedesktop::{ApplicationEntry, ExecuteError, FindError};
use gio::prelude::FileExt;
use thiserror::Error;
use tracing::{error, info};
use waycast_core::LauncherItem;

#[derive(Error, Debug)]
pub enum WaycastError {
    #[error("Failed to launch {0}")]
    AppLaunchError(#[from] ExecuteError),
    #[error("Item with id {0} not found")]
    AppNotFoundError(#[from] FindError),
    #[error("Launch error {0}")]
    LaunchError(String),
}

pub enum Icon {
    ThemeIcon { name: String, path: String },
    Path(String),
}

pub struct WaycastLauncher;

impl WaycastLauncher {
    /// If it's a named theme icon, it will resolve its path.
    /// If it's already an absolute path, it will just return
    /// back the Path variant.
    pub fn resolve_icon(name: &str) -> Option<Icon> {
        // If icon_name is already a path and exists, return it directly
        let path = std::path::Path::new(name);
        if path.exists() {
            return Some(Icon::Path(path.to_string_lossy().to_string()));
        }

        if let Some(path) = freedesktop::get_icon(name) {
            return Some(Icon::ThemeIcon {
                name: name.into(),
                path: path.to_string_lossy().to_string(),
            });
        }

        None
    }

    pub fn execute_item(item: LauncherItem) -> Result<(), WaycastError> {
        match item.kind {
            waycast_core::ItemKind::DesktopEntry => {
                let app = ApplicationEntry::from_id(&item.id)?;
                info!("Found app successfully");
                info!("Path: {}", app.path().display());
                info!("ID: {}", app.id().unwrap_or("Not found".into()));
                app.execute()?;

                Ok(())
            }
            waycast_core::ItemKind::File => {
                info!("Executing: {}", item.id);

                // Use xdg-open directly since it works properly with music files
                // Detach the process so it doesn't die when daemon is killed
                match spawn_detached("xdg-open", &[&item.id]) {
                    Ok(_) => {
                        info!("Successfully launched with xdg-open");
                        Ok(())
                    }
                    Err(e) => {
                        error!("xdg-open failed: {}", e);
                        info!("Attempting GIO method");
                        // Fallback to GIO method
                        let file_gio = gio::File::for_path(&item.id);
                        let ctx = gio::AppLaunchContext::new();
                        match gio::AppInfo::launch_default_for_uri(
                            file_gio.uri().as_str(),
                            Some(&ctx),
                        ) {
                            Ok(()) => {
                                info!("Successfully launched with GIO fallback");
                                Ok(())
                            }
                            Err(e2) => {
                                println!("GIO fallback also failed: {}", e2);
                                Err(WaycastError::LaunchError(e2.to_string()))
                            }
                        }
                    }
                }
            }
            waycast_core::ItemKind::Project => {
                let project_path = item.id;
                let exec_cmd = waycast_config::get::<String>("plugins.projects.open_command")
                    .unwrap_or(String::from("code -n {path}"))
                    .replace("{path}", &project_path);
                let parts: Vec<&str> = exec_cmd.split_whitespace().collect();
                if let Some((program, args)) = parts.split_first() {
                    match spawn_detached(program, args) {
                        Ok(_) => {
                            info!("Successfully opened with configured editor");
                            Ok(())
                        }
                        Err(_) => Err(WaycastError::LaunchError(
                            "Failed to open project folder".into(),
                        )),
                    }
                } else {
                    Err(WaycastError::LaunchError(
                        "No program found in exec_command".into(),
                    ))
                }
            }
            waycast_core::ItemKind::Unknown => todo!(),
        }
    }
}

/// Spawn a detached process that preserves the display environment
use std::process::{Command, Stdio};
pub fn spawn_detached(program: &str, args: &[&str]) -> Result<(), std::io::Error> {
    use std::os::unix::process::CommandExt;

    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // Explicitly preserve important environment variables
    if let Ok(wayland_display) = std::env::var("WAYLAND_DISPLAY") {
        cmd.env("WAYLAND_DISPLAY", wayland_display);
    }
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    if let Ok(xdg_runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        cmd.env("XDG_RUNTIME_DIR", xdg_runtime_dir);
    }
    if let Ok(xdg_session_type) = std::env::var("XDG_SESSION_TYPE") {
        cmd.env("XDG_SESSION_TYPE", xdg_session_type);
    }
    if let Ok(xdg_current_desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        cmd.env("XDG_CURRENT_DESKTOP", xdg_current_desktop);
    }

    unsafe {
        cmd.pre_exec(|| {
            // Start new process group but don't create new session
            // This allows detachment while preserving session environment
            libc::setpgid(0, 0);
            Ok(())
        });
    }

    cmd.spawn()?;

    Ok(())
}
