# Waycast Architecture Refactoring Plan

## Objective
Separate "Waycast" core logic from UI implementation to enable multiple UI types (GTK, terminal, web, etc.) in the future.

## Current State Analysis

Right now `WaycastLauncher` has mixed responsibilities:
- Plugin management and initialization
- Search/filtering logic  
- GTK UI creation and event handling
- Widget rendering and selection

## Proposed Architecture

### 1. Core Launcher Layer (`src/launcher/`)
```rust
pub struct WaycastLauncher {
    plugins: Vec<Arc<dyn LauncherPlugin>>,
    plugins_show_always: Vec<Arc<dyn LauncherPlugin>>,
    plugins_by_prefix: HashMap<String, Arc<dyn LauncherPlugin>>,
    current_results: Vec<Box<dyn LauncherListItem>>,
}

impl WaycastLauncher {
    pub fn new() -> LauncherBuilder { ... }
    pub fn add_plugin(plugin: Box<dyn LauncherPlugin>) -> Self { ... }
    pub fn init_plugins(&self) { ... }
    pub fn get_default_results(&mut self) -> &Vec<Box<dyn LauncherListItem>> { ... }
    pub fn search(&mut self, query: &str) -> &Vec<Box<dyn LauncherListItem>> { ... }
    pub fn execute_item(&self, index: usize) -> Result<(), LaunchError> { ... }
}
```

### 2. UI Abstraction Layer (`src/ui/`)
```rust
pub trait LauncherUI {
    fn show(&self);
    fn hide(&self);
    fn set_results(&mut self, results: &[Box<dyn LauncherListItem>]);
}

pub struct LauncherUIController {
    launcher: WaycastLauncher,
    ui: Box<dyn LauncherUI>,
}
```

### 3. GTK Implementation (`src/ui/gtk/`)
```rust
pub struct GtkLauncherUI {
    window: ApplicationWindow,
    list_view: ListView,
    list_store: ListStore,
    // ... gtk specific fields
}

impl LauncherUI for GtkLauncherUI { ... }
```

### 4. Future Terminal UI (`src/ui/terminal/`)
```rust
pub struct TerminalLauncherUI {
    // crossterm/ratatui components
}

impl LauncherUI for TerminalLauncherUI { ... }
```

## Implementation Plan

### Phase 1: Extract Core Launcher
1. Create `src/launcher/mod.rs` 
2. Move plugin management from `WaycastLauncher` to new `WaycastCore`
3. Move search/filtering logic to core
4. Keep UI-specific code in current location

### Phase 2: Create UI Abstraction  
1. Define `LauncherUI` trait
2. Create `LauncherUIController` to coordinate core + UI
3. Update `main.rs` to use controller pattern

### Phase 3: Refactor GTK Implementation
1. Move GTK code to `src/ui/gtk/`
2. Implement `LauncherUI` trait for GTK
3. Remove UI logic from core launcher

### Phase 4: Clean Interface
1. Define clean events/callbacks between core and UI
2. Handle UI -> Core communication (search input, item selection)
3. Handle Core -> UI communication (results updates, state changes)

## Key Benefits

- **Plugin logic** stays UI-agnostic
- **Search/filtering** can be unit tested without UI
- **Multiple UIs** can share the same core
- **Clear separation** of data vs presentation
- **Future expansion** for web UI, CLI, etc.

## Implementation Status
- [x] Phase 1: Extract Core Launcher
- [x] Phase 2: Create UI Abstraction  
- [x] Phase 3: Refactor GTK Implementation
- [x] Phase 4: Clean Interface