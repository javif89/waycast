# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WayCast is a Raycast-like application launcher for Linux, built with C++20 and Qt6. It scans XDG desktop entries to provide a searchable interface for launching applications on Wayland/X11.

## Build System

The project supports three build systems:

### Primary: CMake with Ninja (Recommended)
```bash
# Configure and build
make configure  # or: cmake -S . -B build -G Ninja
make bld       # or: cmake --build build
make run       # or: ./build/waycast

# Combined: make br (build + run)
# Full rebuild: make all (configure + build + run)
```

### Alternative: Zig Build
```bash
zig build run
```

### Development Environment: Nix Shell
```bash
nix-shell  # Sets up Qt6, CMake, Ninja, Clang, and required dependencies
```

## Architecture

### Core Components

- **main.cpp**: Entry point, currently configured for CLI testing of desktop entry parsing
- **dmenu.hpp/namespace**: Desktop entry parsing using GIO/GLib to read XDG application data
- **files.hpp/namespace**: File system utilities for scanning directories and reading files
- **ui/Main.qml**: Qt Quick interface (currently minimal, Qt GUI code commented out in main)

### Key Classes

- `dmenu::DesktopEntry`: Parses .desktop files using GDesktopAppInfo, extracts app metadata (name, icon, executable, display flags)
- `files::findFilesWithExtension()`: Recursively scans directories for files with specific extensions
- `DEVec`: Type alias for `std::unique_ptr<std::vector<DesktopEntry>>`

### Current State

The application is in active development:
- Main Qt GUI loop is commented out in main.cpp:56-64
- Currently runs as CLI tool that prints discovered application IDs
- Desktop entry scanning logic is functional
- Qt QML interface exists but is not connected

## Dependencies

- **Qt6**: Core, Gui, Qml, Quick, QuickControls2
- **GIO/GLib**: For XDG desktop entry parsing
- **C++20**: Uses std::format, filesystem, and modern C++ features
- **CMake 3.21+**: Build system
- **Ninja**: Preferred generator

## Development Notes

- Qt resources are bundled via CMakeLists.txt (qt_add_qml_module)
- Uses C++20 modules compilation flags for GCC/Clang
- Nix shell provides complete development environment with Qt6 Wayland support
- Built-in RPATH configuration for Linux runtime library discovery

## Project Conventions

- We're not using header files. Prioritize .hpp