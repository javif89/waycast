# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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