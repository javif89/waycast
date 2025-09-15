use std::process::{Command, Stdio};

/// Spawn a detached process that won't be killed when the parent daemon dies
pub fn spawn_detached(program: &str, args: &[&str]) -> Result<(), std::io::Error> {
    use std::os::unix::process::CommandExt;

    let mut cmd = Command::new(program);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    unsafe {
        cmd.pre_exec(|| {
            // Start new session - this detaches from parent's process group
            libc::setsid();
            Ok(())
        });
    }

    cmd.spawn()?;

    Ok(())
}
