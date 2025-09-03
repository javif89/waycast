# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Waycast is a GTK4-based application launcher for Wayland compositors, built with Rust using the relm4 framework. It provides a floating launcher interface that displays desktop applications with icons and allows users to search and launch them.

## Build and Development Commands

```bash
# Build the project
cargo build

# Run the application
cargo run

# Build for release
cargo build --release

# Check code formatting
cargo fmt --check

# Format code
cargo fmt

# Run clippy lints
cargo clippy

# Run tests (if any exist)
cargo test
```

## Architecture

### Core Components

- **main.rs**: Contains the main GTK4/relm4 application with two primary components:
  - `AppModel`: Main window component with search functionality
  - `ListItem`: Factory component for rendering individual launcher items

- **lib.rs**: Defines core traits and types:
  - `LauncherListItem` trait: Interface for launchable items
  - `LaunchError` enum: Error handling for launch operations

- **drun module** (`src/drun/mod.rs`): Handles desktop application discovery
  - `DesktopEntry` struct: Represents a .desktop file
  - `all()` function: Scans XDG_DATA_DIRS for desktop applications
  - Implements `LauncherListItem` trait for desktop entries

- **util module** (`src/util/`): Utility functions
  - `files.rs`: File system operations, particularly `get_files_with_extension()`

### Key Technologies

- **GTK4**: UI framework with gtk4-layer-shell for Wayland layer shell protocol
- **relm4**: Reactive UI framework for GTK4 applications
- **gio**: GLib I/O library for desktop app info and icon handling

### Important Implementation Details

- Uses gtk4-layer-shell to create a floating overlay window on Wayland
- Desktop applications are discovered by parsing .desktop files from XDG_DATA_DIRS
- Icons are handled through GIO's Icon system (ThemedIcon and FileIcon)
- Factory pattern is used for efficiently rendering lists of launcher items

### Lifetime Management

When working with GTK widgets in relm4 view macros, be careful with string references. The view macro context has specific lifetime requirements:
- Avoid returning `&str` from methods called in view macros
- Use `self.field.as_ref().map(|s| s.as_str())` pattern for Option<String> to Option<&str> conversion
- Static strings work fine, but dynamic references may cause stack overflows

### Module Structure

```
src/
├── main.rs           # Main application and UI components
├── lib.rs            # Core traits and error types
├── drun/
│   └── mod.rs        # Desktop application discovery
└── util/
    ├── mod.rs        # Utility module exports
    └── files.rs      # File system utilities
```