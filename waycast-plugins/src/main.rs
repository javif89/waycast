use std::{collections::BTreeMap, path::PathBuf};
use tokei::{Config, LanguageType, Languages};
use waycast_plugins::projects::{
    framework_detector::{self, FrameworkDetector},
    type_scanner::TypeScanner,
};

pub fn main() {
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

            if let Some(name) = fw {
                println!("{}: {}", e.path().display(), name);
            } else {
                println!("{}: {}", e.path().display(), "NONE");
            }
            // let langs = scanner.scan(e.path(), Some(3));
            // // let langs = lang_breakdown(&[e.path().to_str().unwrap()], &[]);

            // let top: Vec<String> = langs.iter().map(|l| l.name.to_owned()).collect();

            // println!("{}: {:?}", e.path().display(), top);
        }
    }
}
