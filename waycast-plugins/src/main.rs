use std::{collections::BTreeMap, path::PathBuf};
use tokei::{Config, LanguageType, Languages};
use waycast_plugins::projects::{
    framework_detector::{self, FrameworkDetector},
    type_scanner::TypeScanner,
};

struct Project {
    project_type: String,
    path: PathBuf,
}
pub fn main() {
    let mut projects: Vec<Project> = Vec::new();
    let scanner = TypeScanner::new();
    let framework_detector = FrameworkDetector::new();
    if let Ok(entries) = std::fs::read_dir(PathBuf::from("/home/javi/projects")) {
        for e in entries
            .into_iter()
            .filter(|e| e.is_ok())
            .map(|e| e.unwrap())
            .filter(|e| e.path().is_dir())
        {
            let fw = framework_detector.detect(e.path().to_string_lossy().to_string().as_str());

            let mut project_type: String = String::from("NONE");

            if let Some(name) = fw {
                project_type = name;
            } else {
                let langs = scanner.scan(e.path(), Some(1));
                if let Some(l) = langs.first() {
                    project_type = l.name.to_owned()
                }
            }

            projects.push(Project {
                project_type,
                path: e.path().to_path_buf(),
            });
        }
    }

    for p in projects {
        println!("{}: {}", p.path.display(), p.project_type);
    }
}
