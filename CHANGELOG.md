# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] - 2025-09-16

### Breaking Changes
- **Architecture Revert**: Removed daemon-based architecture and returned to direct embedded launcher
- **Removed Components**: Eliminated `waycast-daemon` and `waycast-protocol` crates

### Added
- **Direct UI Integration**: GTK application now constructs its own `WaycastLauncher` instance directly
- **All Plugin Support**: Restored support for all plugins (drun, file_search, projects) in single process

### Fixed
- **Application Spawn Environment**: Fixed critical issue where launched applications lost display environment access
- **VS Code Terminal Compatibility**: Applications launched through waycast now properly inherit display variables
- **Session Preservation**: Changed from `setsid()` to `setpgid(0, 0)` to maintain session while detaching processes

### Enhanced
- **Development Experience**: Simplified architecture makes debugging and development easier

### Technical
- **Spawn Function Improvements**: Enhanced `spawn_detached` to explicitly preserve `WAYLAND_DISPLAY`, `DISPLAY`, `XDG_RUNTIME_DIR`, `XDG_SESSION_TYPE`, and `XDG_CURRENT_DESKTOP`
- **Nix Development Shell**: Added proper display environment variable setup for VS Code integration
- **Process Group Management**: Proper detachment without losing session context

### Infrastructure
- **Systemd Removal**: Eliminated systemd service installation and management

## [0.1.1] - 2025-09-15

### Enhanced

**UI Startup Performance**: Loading icons async and caching results makes the UI load faster.

## [0.1.0] - 2025-09-15

This update introduces the waycast-daemon binary. Instead of re-doing all the computation of starting up plugins,
scanning the file system and getting desktop entries every time the app starts, this will now happen in a long running
background process.

### Breaking Changes
- **Architecture Refactor**: Moved from embedded launcher to daemon-based architecture

### Added
- **Waycast Daemon**: Background service for managing launcher state and plugins
- **JSON-RPC Protocol**: Simple, debuggable communication protocol between GTK client and daemon
- **Automatic Refresh**: Daemon periodically rescans applications and projects every 2 minutes
- **Systemd Integration**: User service configuration for automatic daemon startup
- **Home Manager Support**: Enhanced module with optional daemon service management

### Enhanced
- **Performance**: Persistent daemon keeps data indexed and ready. Should decrease UI startup time since launcher logic now lives in a separate long running process.

### Technical
- **Protocol Crates**: New `waycast-protocol` crate with client/server libraries
- **Socket Utilities**: Automatic socket path resolution with fallbacks
- **Error Handling**: Comprehensive protocol error types with proper propagation
- **Thread Safety**: Arc/Mutex patterns for safe daemon state sharing
- **Background Tasks**: Tokio-based async server with sync client compatibility

### Infrastructure
- **Service Definition**: Systemd user service file with proper dependencies
- **Nix Integration**: Flake automatically installs systemd service configuration
- **Build System**: Updated workspace with new protocol crate

## [0.0.3] - 2025-09-12

### Fixed
- **Home manager module**: The icons were not getting properly copied to $XDG_DATA_HOME

## [0.0.2] - 2025-09-11

### Added
- **Framework Detection System**: Automatic detection of project types (Laravel, Rails, NextJS, Vue, Svelte, Django, Flask, Go Fiber, Ansible)
- **Project Icons**: Visual framework/language icons for project entries using devicons
- **Intelligent Project Type Detection**: Uses both framework detection and language analysis (tokei) with fallback logic
- **Icon Asset Management**: Bundled devicon SVG assets with proper XDG data directory support
- **Caching System**: 24-hour TTL caching for project type detection to improve performance
- **Development Cache Control**: `WAYCAST_NO_CACHE` environment variable to disable caching during development
- **Macro-based Framework Registration**: Clean, declarative system for adding new framework detectors
- **Asset Installation**: Makefile targets for icon management and XDG directory installation
- **Release Management**: Automated release workflow with version bumping across all crates

### Enhanced
- **Project Plugin**: Now shows framework/language-specific icons instead of generic VSCode icon
- **Icon Resolution**: Smart path resolution with XDG directory precedence and development fallback
- **Nix Package**: Added icon installation to flake.nix build process

### Technical
- **Framework Detection Macro**: Compile-time static framework detection system with zero runtime overhead
- **JSON Path Checking**: Built-in support for detecting frameworks via package.json dependencies
- **File Pattern Matching**: Flexible file existence and directory structure checking
- **Custom Validation**: Support for complex framework detection logic via closures

## [0.0.1] - 2025-0908-

### Added
- **Core Application**: GTK4-based application launcher for Wayland compositors
- **Plugin System**: Modular plugin architecture with priority-based ordering
- **Desktop Application Plugin**: Scan and launch .desktop applications from XDG directories
- **File Search Plugin**: Fast file system search with configurable directories and exclusions
- **Project Search Plugin**: Code project discovery with configurable search paths
- **Configuration System**: TOML-based configuration with environment variable support
- **Caching Framework**: Persistent disk cache with TTL support using redb
- **Wayland Integration**: Layer shell support for floating launcher interface
- **Search Interface**: Type-to-filter functionality with instant results
- **Nix Support**: Complete Nix flake with development shell and package definition
- **Home Manager Module**: NixOS home-manager integration for declarative configuration

### Technical
- **Multi-crate Architecture**: Organized codebase with core, plugins, config, and UI separation  
- **Async Plugin Loading**: Background indexing with non-blocking UI
- **XDG Compliance**: Proper XDG Base Directory specification support
- **GTK4 + Relm4**: Modern reactive UI framework with factory patterns for list rendering
- **Icon Handling**: GIO-based icon resolution with themed and file icon support
- **Error Handling**: Comprehensive error types with proper propagation
- **Memory Efficiency**: Static compilation patterns and lazy initialization
- **Development Tooling**: Comprehensive Makefile with build, test, and quality targets

### Infrastructure
- **Build System**: Cargo workspace with proper dependency management
- **Development Environment**: Nix development shell with all required dependencies
- **Code Quality**: Clippy linting, rustfmt formatting, and automated git hooks
- **Documentation**: Inline documentation and architectural guidance in CLAUDE.md