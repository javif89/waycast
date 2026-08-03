use crate::core::LauncherItem;
use freedesktop::{ApplicationEntry, ExecuteError, FindError};
use gio::prelude::FileExt;
use thiserror::Error;
use tracing::{error, info};

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
            crate::core::ItemKind::DesktopEntry => {
                let app = ApplicationEntry::from_id(&item.id)?;
                info!("Found app successfully");
                info!("Path: {}", app.path().display());
                info!("ID: {}", app.id().unwrap_or("Not found".into()));

                let (cmd, args) = app.prepare_command(&[], &[])?;
                info!("Executing with command: {} | args: {}", cmd, args.join(","));
                let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
                let working_dir = app.path_dir();
                let opts = SpawnOptions {
                    working_dir: working_dir.as_deref(),
                    scope_id: Some(&item.id),
                };
                match spawn_detached(&cmd, &arg_refs, opts) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(WaycastError::LaunchError(e.to_string())),
                }
            }
            crate::core::ItemKind::File => {
                info!("Executing: {}", item.id);

                // Use xdg-open directly since it works properly with music files
                // Detach the process so it doesn't die when daemon is killed
                match spawn_detached("xdg-open", &[&item.id], SpawnOptions::default()) {
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
            crate::core::ItemKind::Project => {
                let project_path = item.id;
                let exec_cmd = crate::core::config::get::<String>("plugins.projects.open_command")
                    .unwrap_or(String::from("code -n {path}"))
                    .replace("{path}", &project_path);
                let parts: Vec<&str> = exec_cmd.split_whitespace().collect();
                if let Some((program, args)) = parts.split_first() {
                    match spawn_detached(program, args, SpawnOptions::default()) {
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
            crate::core::ItemKind::Unknown => todo!(),
        }
    }
}

use std::process::{Command, Stdio};
use std::sync::OnceLock;

/// Options controlling how a process is spawned.
///
/// The launched process inherits waycast's environment, so nothing here needs
/// to forward `WAYLAND_DISPLAY` and friends by hand.
#[derive(Default)]
pub struct SpawnOptions<'a> {
    /// Working directory, e.g. a desktop entry's `Path=` key.
    pub working_dir: Option<&'a str>,
    /// Identifier the systemd scope is named after, so launched apps are
    /// recognizable in `systemd-cgls` and process monitors. Defaults to the
    /// program name.
    pub scope_id: Option<&'a str>,
}

/// Whether launched processes can be placed in their own systemd scope.
///
/// Reparenting a process does not move it out of the daemon's cgroup, so
/// without a scope of its own every app stays subject to the daemon unit's
/// `KillMode` and dies whenever the daemon restarts.
///
/// The probe actually starts a throwaway scope rather than sniffing for
/// `systemd-run` on `PATH`: launches are double-forked, which makes the real
/// call's exit status unobservable, so it has to be proven to work up front.
fn systemd_scopes_available() -> bool {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();

    *AVAILABLE.get_or_init(|| {
        let available = Command::new("systemd-run")
            .args(["--user", "--scope", "--collect", "--quiet", "--", "true"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success());

        if available {
            info!("Launching apps in transient systemd scopes");
        } else {
            info!("systemd-run unavailable, launching apps without cgroup isolation");
        }

        available
    })
}

/// Build a unique, readable transient unit name for a launch.
fn scope_unit_name(id: &str) -> String {
    let base = std::path::Path::new(id)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| id.to_owned());
    let base = base.strip_suffix(".desktop").unwrap_or(&base);

    let mut sanitized: String = base
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        .take(64)
        .collect();
    if sanitized.is_empty() {
        sanitized.push_str("app");
    }

    // systemd rejects duplicate unit names, so launching the same app twice
    // must not collide.
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.subsec_nanos());

    format!("app-waycast-{sanitized}-{nonce:08x}")
}

/// Spawn a process fully detached from waycast.
///
/// Detached means three things:
/// the process leaves waycast's process tree.
/// leaves its session, and gets reaped rather than left behind as a zombie.
/// Where systemd is available it also lands in a cgroup of its own.
pub fn spawn_detached(
    program: &str,
    args: &[&str],
    opts: SpawnOptions<'_>,
) -> Result<(), std::io::Error> {
    use std::os::unix::process::CommandExt;

    let mut cmd = if systemd_scopes_available() {
        let unit = scope_unit_name(opts.scope_id.unwrap_or(program));
        let mut cmd = Command::new("systemd-run");
        cmd.args(["--user", "--scope", "--collect", "--quiet"])
            .arg(format!("--unit={unit}"))
            .arg("--")
            .arg(program)
            .args(args);
        cmd
    } else {
        let mut cmd = Command::new(program);
        cmd.args(args);
        cmd
    };

    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    if let Some(dir) = opts.working_dir {
        cmd.current_dir(dir);
    }

    unsafe {
        cmd.pre_exec(|| {
            // Leave the daemon's session and controlling terminal. Without
            // this, anything using PR_SET_PDEATHSIG (bubblewrap's
            // --die-with-parent, as steam does) dies with the daemon.
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }

            // Fork again so the process that execs is orphaned and reparented
            // away from waycast. setpgid alone only changed the process group,
            // which left every app a child of the daemon.
            match libc::fork() {
                -1 => Err(std::io::Error::last_os_error()),
                0 => Ok(()),         // grandchild: go on to exec
                _ => libc::_exit(0), // intermediate: vanish
            }
        });
    }

    // The intermediate exits immediately, so this returns right away and reaps
    // it instead of leaving a zombie parked under the daemon.
    cmd.spawn()?.wait()?;

    Ok(())
}
