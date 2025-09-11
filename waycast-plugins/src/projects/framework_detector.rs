use std::path::{Path, PathBuf};

pub enum Framework {
    Laravel,
    Rails,
    Vue,
    NextJS,
    Ansible,
}

fn has_file<P: AsRef<Path>>(project_path: P, file: P) -> bool {
    PathBuf::from(project_path.as_ref()).join(file).exists()
}

fn read_json_config<P: AsRef<Path>>(project_path: P, file: P) -> Option<serde_json::Value> {
    let pb = PathBuf::from(project_path.as_ref());
    if let Ok(text) = std::fs::read_to_string(pb.join(file)) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            return Some(v);
        }
    } else {
        return None;
    }

    None
}

trait FrameworkHeuristics: Sync {
    fn name(&self) -> &'static str;
    fn matches(&self, project_path: &str) -> bool;
}

struct Laravel;
impl FrameworkHeuristics for Laravel {
    fn name(&self) -> &'static str {
        "Laravel"
    }

    fn matches(&self, project_path: &str) -> bool {
        // Check for composer.json
        if !has_file(project_path, "composer.json") {
            return false;
        }

        // If composer.json has "laravel/framework"
        // we can say yes immediately
        if let Some(cfg) = read_json_config(project_path, "composer.json") {
            let requires_laravel = cfg
                .get("require")
                .and_then(|r| r.get("laravel/framework"))
                .is_some();
            if requires_laravel {
                return true;
            }
        }

        false
    }
}

pub struct FrameworkDetector {
    heuristics: &'static [&'static dyn FrameworkHeuristics],
}

static LARAVEL: Laravel = Laravel;
static HEURISTICS: &[&dyn FrameworkHeuristics] = &[&LARAVEL];

impl FrameworkDetector {
    pub fn new() -> FrameworkDetector {
        FrameworkDetector {
            heuristics: HEURISTICS,
        }
    }

    pub fn detect(&self, project_path: &str) -> Option<String> {
        for h in self.heuristics {
            if h.matches(project_path) {
                return Some(String::from(h.name()));
            }
        }

        None
    }
}
