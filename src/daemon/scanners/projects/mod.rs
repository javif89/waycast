pub mod framework_detector;
pub mod framework_macro;
pub mod type_scanner;
use std::{collections::HashSet, fs, path::PathBuf};

use crate::core::{LauncherItem, WaycastScanner};
use std::sync::LazyLock;

use framework_detector::FrameworkDetector;
use type_scanner::TypeScanner;

static TOKEI_SCANNER: LazyLock<TypeScanner> = LazyLock::new(TypeScanner::new);
static FRAMEWORK_DETECTOR: LazyLock<FrameworkDetector> = LazyLock::new(FrameworkDetector::new);

#[derive(Clone)]
pub struct ProjectEntry {
    path: PathBuf,
    project_type: Option<String>,
}

fn get_icon(p: &ProjectEntry) -> String {
    if let Some(t) = &p.project_type {
        // Try XDG data directory first, fall back to development path
        let icon_name = format!("{}.svg", t.to_lowercase());

        if let Some(data_dir) = crate::core::config::data_dir() {
            let xdg_icon_path = data_dir.join("icons").join("devicons").join(&icon_name);
            if xdg_icon_path.exists() {
                return xdg_icon_path.to_string_lossy().to_string();
            }
        }

        // Fall back to development path
        let dev_icon_path = PathBuf::from("./assets/icons/devicons").join(&icon_name);
        if dev_icon_path.exists() {
            return dev_icon_path.to_string_lossy().to_string();
        }
    }

    String::from("vscode")
}

impl From<ProjectEntry> for LauncherItem {
    fn from(val: ProjectEntry) -> Self {
        LauncherItem {
            id: val.path.to_string_lossy().to_string(),
            title: String::from(val.path.file_name().unwrap().to_string_lossy()),
            kind: crate::core::ItemKind::Project,
            description: Some(val.path.to_string_lossy().to_string()),
            icon: get_icon(&val),
        }
    }
}

pub struct ProjectScanner {
    search_paths: HashSet<PathBuf>,
}

impl Default for ProjectScanner {
    fn default() -> Self {
        Self::new(HashSet::new())
    }
}

impl ProjectScanner {
    pub fn new(paths: HashSet<PathBuf>) -> Self {
        Self {
            search_paths: paths,
        }
    }

    pub fn get_search_paths(&self) -> HashSet<PathBuf> {
        self.search_paths.clone()
    }
}

impl WaycastScanner for ProjectScanner {
    fn scan(&self) -> Vec<LauncherItem> {
        let mut project_entries = Vec::new();

        for search_path in &self.search_paths {
            if let Ok(entries) = fs::read_dir(search_path) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type()
                        && file_type.is_dir()
                    {
                        let path = entry.path();

                        // Skip hidden directories (starting with .)
                        if let Some(file_name) = path.file_name()
                            && let Some(name_str) = file_name.to_str()
                        {
                            // Skip hidden directories
                            if name_str.starts_with('.') {
                                continue;
                            }

                            let project_type =
                                detect_project_type(path.to_string_lossy().to_string().as_str());
                            project_entries.push(ProjectEntry { path, project_type });
                        }
                    }
                }
            }
        }

        project_entries
            .iter()
            .map(|p| p.to_owned().into())
            .collect()
    }
}

fn detect_project_type(path: &str) -> Option<String> {
    let detect_fn = |path| {
        let fw = FRAMEWORK_DETECTOR.detect(path);
        if let Some(name) = fw {
            return Some(name);
        } else {
            let langs = TOKEI_SCANNER.scan(path, Some(1));
            if let Some(l) = langs.first() {
                // We do some special replacements so it's easier to match
                // with the icon file names
                return Some(l.name.to_owned().replace("+", "plus").replace("#", "sharp"));
            }
        }

        None
    };

    detect_fn(path)
}
