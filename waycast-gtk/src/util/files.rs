use std::path::{Path, PathBuf};
use std::{fs, io};

pub fn get_files_with_extension<P: AsRef<Path>>(
    dir: P,
    extension: &str,
) -> io::Result<Vec<PathBuf>> {
    let entries = fs::read_dir(dir)?;
    let desktop_files: Vec<_> = entries
        .filter_map(|res| res.ok())
        .map(|f| f.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == extension)
                .unwrap_or(false)
        })
        .collect();

    let mut files = Vec::new();
    for f in desktop_files {
        files.push(f);
    }

    Ok(files)
}
