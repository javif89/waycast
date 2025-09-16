use std::process::{Command, Stdio};

/// Spawn a detached process that preserves the display environment
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
