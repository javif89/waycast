# Waycast Macros

Procedural macros for the Waycast launcher framework to reduce boilerplate when implementing plugins and launcher items.

## Macros

### `plugin!`

Generates `LauncherPlugin` trait method implementations inside an `impl LauncherPlugin` block.

#### Usage

```rust
use waycast_core::{LauncherPlugin, LauncherListItem};
use waycast_macros::plugin;

pub struct MyPlugin {
    // Your custom fields
    data: Vec<String>,
}

impl MyPlugin {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    
    // Your custom methods
    pub fn add_item(&mut self, item: String) {
        self.data.push(item);
    }
}

impl LauncherPlugin for MyPlugin {
    plugin! {
        name: "My Plugin",
        priority: 500,
        description: "A sample plugin",
        prefix: "my",
        by_prefix_only: false,
        init: my_init,
        default_list: my_default_list,
        filter: my_filter
    }
}

// Implement your plugin functions
fn my_init(plugin: &MyPlugin) {
    println!("Initializing plugin with {} items", plugin.data.len());
}

fn my_default_list(plugin: &MyPlugin) -> Vec<Box<dyn LauncherListItem>> {
    // Return default items
    Vec::new()
}

fn my_filter(plugin: &MyPlugin, query: &str) -> Vec<Box<dyn LauncherListItem>> {
    // Filter and return matching items
    Vec::new()
}
```

#### Parameters

- `name`: **Required** - String literal for plugin name
- `priority`: Optional - Integer priority (default: 100)
- `description`: Optional - String description
- `prefix`: Optional - String prefix for queries
- `by_prefix_only`: Optional - Boolean, whether plugin only responds to prefix queries (default: false)
- `init`: Optional - Function name for initialization
- `default_list`: Optional - Function name that returns default items
- `filter`: Optional - Function name that filters items based on query

#### Function Signatures

```rust
// All functions are optional
fn init_function(plugin: &YourPluginType) {
    // Initialize plugin
}

fn default_list_function(plugin: &YourPluginType) -> Vec<Box<dyn LauncherListItem>> {
    // Return default items when no query
}

fn filter_function(plugin: &YourPluginType, query: &str) -> Vec<Box<dyn LauncherListItem>> {
    // Return filtered items based on query
}
```

### `launcher_entry!`

Generates `LauncherListItem` trait method implementations inside an `impl LauncherListItem` block.

#### Usage

```rust
use waycast_core::{LaunchError, LauncherListItem};
use waycast_macros::launcher_entry;

#[derive(Clone)]
pub struct MyItem {
    name: String,
    path: PathBuf,
}

impl LauncherListItem for MyItem {
    launcher_entry! {
        id: format!("item_{}", self.name),
        title: self.name.clone(),
        description: Some(format!("Item at {}", self.path.display())),
        icon: "application-x-executable".to_string(),
        execute: {
            println!("Executing {}", self.name);
            std::process::Command::new("xdg-open")
                .arg(&self.path)
                .spawn()
                .map_err(|e| LaunchError::CouldNotLaunch(e.to_string()))?;
            Ok(())
        }
    }
}
```

#### Parameters

- `id`: **Required** - Expression returning `String` unique identifier
- `title`: **Required** - Expression returning `String` display title  
- `description`: Optional - Expression returning `Option<String>` description
- `icon`: **Required** - Expression returning `String` icon name or path
- `execute`: **Required** - Expression returning `Result<(), LaunchError>` execution logic

#### Expression Types

All parameters accept Rust expressions:

**Simple expressions:**
```rust
id: self.name.clone(),
title: "My Title".to_string(),
```

**Complex expressions with blocks:**
```rust
icon: {
    if self.is_directory() {
        "folder".to_string()
    } else {
        "file".to_string()
    }
},
execute: {
    println!("Opening {}", self.path.display());
    // Complex execution logic
    match std::process::Command::new("xdg-open").arg(&self.path).spawn() {
        Ok(_) => Ok(()),
        Err(e) => Err(LaunchError::CouldNotLaunch(e.to_string())),
    }
}
```

## Examples

### Simple Plugin Example

```rust
use waycast_macros::{plugin, launcher_entry};
use waycast_core::{LaunchError, LauncherListItem, LauncherPlugin};

// Simple plugin with no state
pub struct CalculatorPlugin;

impl CalculatorPlugin {
    pub fn new() -> Self {
        CalculatorPlugin
    }
}

impl LauncherPlugin for CalculatorPlugin {
    plugin! {
        name: "Calculator",
        priority: 800,
        description: "Perform calculations",
        prefix: "calc"
    }
}

// Simple item
#[derive(Clone)]
struct CalcResult {
    expression: String,
    result: f64,
}

impl LauncherListItem for CalcResult {
    launcher_entry! {
        id: self.expression.clone(),
        title: format!("{} = {}", self.expression, self.result),
        description: Some("Calculation result".to_string()),
        icon: "accessories-calculator".to_string(),
        execute: {
            println!("Result: {}", self.result);
            Ok(())
        }
    }
}
```

### Complex Plugin Example

```rust
use waycast_macros::{plugin, launcher_entry};
use waycast_core::{LaunchError, LauncherListItem, LauncherPlugin};
use std::path::PathBuf;

// Complex plugin with state and custom methods
pub struct FileSearchPlugin {
    search_paths: Vec<PathBuf>,
    max_results: usize,
}

impl FileSearchPlugin {
    pub fn new() -> Self {
        Self {
            search_paths: vec![PathBuf::from("/home")],
            max_results: 50,
        }
    }
    
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }
    
    pub fn set_max_results(&mut self, max: usize) {
        self.max_results = max;
    }
}

impl LauncherPlugin for FileSearchPlugin {
    plugin! {
        name: "File Search",
        priority: 600,
        description: "Search and open files",
        prefix: "file",
        init: file_search_init,
        filter: file_search_filter
    }
}

fn file_search_init(plugin: &FileSearchPlugin) {
    println!("Initialized file search with {} paths", plugin.search_paths.len());
}

fn file_search_filter(plugin: &FileSearchPlugin, query: &str) -> Vec<Box<dyn LauncherListItem>> {
    // Implementation would search files and return FileEntry items
    Vec::new()
}

// Complex item with file operations
#[derive(Clone)]
struct FileEntry {
    path: PathBuf,
}

impl LauncherListItem for FileEntry {
    launcher_entry! {
        id: self.path.to_string_lossy().to_string(),
        title: self.path.file_name().unwrap().to_string_lossy().to_string(),
        description: Some(format!("File: {}", self.path.display())),
        icon: {
            // Complex icon detection logic
            let extension = self.path.extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            match extension {
                "txt" | "md" => "text-x-generic",
                "png" | "jpg" | "jpeg" => "image-x-generic",
                "mp3" | "wav" | "flac" => "audio-x-generic",
                _ => "application-x-generic"
            }.to_string()
        },
        execute: {
            println!("Opening file: {}", self.path.display());
            std::process::Command::new("xdg-open")
                .arg(&self.path)
                .spawn()
                .map_err(|e| LaunchError::CouldNotLaunch(format!("Failed to open file: {}", e)))?;
            Ok(())
        }
    }
}
```

## IDE Support

Both macros include built-in rust-analyzer support to prevent "trait not fully implemented" errors in your editor. The macros automatically generate stub implementations that are only visible to rust-analyzer, ensuring a smooth development experience.

## Requirements

- Rust 2021 edition or later
- `waycast-core` crate for trait definitions
- `syn`, `quote`, and `proc-macro2` dependencies (handled automatically)