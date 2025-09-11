use std::path::{Path, PathBuf};

pub fn has_file<P: AsRef<Path>>(project_path: P, file: P) -> bool {
    PathBuf::from(project_path.as_ref()).join(file).exists()
}

pub fn has_directory<P: AsRef<Path>>(project_path: P, dir: P) -> bool {
    let path = PathBuf::from(project_path.as_ref()).join(dir);
    path.exists() && path.is_dir()
}

pub fn read_json_config<P: AsRef<Path>>(project_path: P, file: P) -> Option<serde_json::Value> {
    let pb = PathBuf::from(project_path.as_ref());
    if let Ok(text) = std::fs::read_to_string(pb.join(file)) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            return Some(v);
        }
    }
    None
}

pub fn check_json_path(json: &serde_json::Value, path: &str) -> bool {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = json;
    
    for part in parts {
        if let Some(next) = current.get(part) {
            current = next;
        } else {
            return false;
        }
    }
    
    true
}

pub trait FrameworkHeuristics: Sync {
    fn name(&self) -> &'static str;
    fn matches(&self, project_path: &str) -> bool;
}

#[macro_export]
macro_rules! frameworks {
    (
        $(
            $name:ident {
                $(files: [$($file:literal),* $(,)?],)?
                $(directories: [$($dir:literal),* $(,)?],)?
                $(json_checks: [$(($json_file:literal, $json_path:literal)),* $(,)?],)?
                $(custom: $custom_fn:expr,)?
            }
        ),* $(,)?
    ) => {
        $(
            struct $name;
            impl $crate::projects::framework_macro::FrameworkHeuristics for $name {
                fn name(&self) -> &'static str {
                    stringify!($name)
                }
                
                fn matches(&self, project_path: &str) -> bool {
                    // Check required files first
                    $(
                        $(
                            if !$crate::projects::framework_macro::has_file(project_path, $file) {
                                return false;
                            }
                        )*
                    )?
                    
                    // Check directories - any match returns true
                    $(
                        $(
                            if $crate::projects::framework_macro::has_directory(project_path, $dir) {
                                return true;
                            }
                        )*
                    )?
                    
                    // Check JSON paths - any match returns true
                    $(
                        $(
                            if let Some(json) = $crate::projects::framework_macro::read_json_config(project_path, $json_file) {
                                if $crate::projects::framework_macro::check_json_path(&json, $json_path) {
                                    return true;
                                }
                            }
                        )*
                    )?
                    
                    // Custom validation
                    $(
                        if ($custom_fn)(project_path) {
                            return true;
                        }
                    )?
                    
                    // If we have files specified but no other checks, files existing means match
                    #[allow(unreachable_code)]
                    {
                        $(
                            $(
                                let _ = $file; // Use the file variable to indicate files were specified
                                return true;
                            )*
                        )?
                        false
                    }
                }
            }
            
        )*
        
        static HEURISTICS: &[&dyn $crate::projects::framework_macro::FrameworkHeuristics] = &[
            $(&$name {},)*
        ];
    };
}