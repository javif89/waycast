use std::path::{Path, PathBuf};
use crate::protocol::{ProtocolError, Result};

pub fn default_socket_path() -> Result<PathBuf> {
    // Try user-specific socket in /tmp first
    let uid = unsafe { libc::getuid() };
    let tmp_path = PathBuf::from(format!("/tmp/waycast-daemon-{}.sock", uid));
    
    // Check if /tmp is writable
    if Path::new("/tmp").exists() {
        return Ok(tmp_path);
    }
    
    // Fallback to user config directory
    let config_dir = dirs::config_dir()
        .ok_or_else(|| ProtocolError::Connection(
            std::io::Error::new(std::io::ErrorKind::NotFound, "No config directory found")
        ))?;
    
    let waycast_dir = config_dir.join("waycast");
    std::fs::create_dir_all(&waycast_dir)?;
    
    Ok(waycast_dir.join("daemon.sock"))
}

pub fn cleanup_socket(path: &Path) -> Result<()> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_socket_path() {
        let path = default_socket_path().unwrap();
        assert!(path.to_string_lossy().contains("waycast"));
    }
}