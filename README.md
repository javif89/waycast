# WayCast

A Raycast-like application launcher for Linux with Qt6 and Wayland support. We deserve this.

## Features

- **Fast fuzzy search** with AVX2 acceleration
- **Desktop application discovery** via XDG desktop entries
- **File search** with path-based filtering (e.g., `wallpapers/sunset.jpg`)
- **Plugin system** for extensible functionality
- **Modern UI** with Material Dark theme
- **Wayland native** with layer shell support
- **Semi-transparent** window

## Installation

### Quick Install (Nix)

```bash
nix profile install git+https://gitgud.foo/thegrind/waycast
```

### Run Without Installing

```bash
nix run git+https://gitgud.foo/thegrind/waycast
```

### NixOS System-Wide Installation

Add to your `flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    waycast.url = "git+https://gitgud.foo/thegrind/waycast";
  };

  outputs = { nixpkgs, waycast, ... }: {
    nixosConfigurations.yourhost = nixpkgs.lib.nixosSystem {
      modules = [
        waycast.nixosModules.waycast
        {
          programs.waycast.enable = true;
        }
      ];
    };
  };
}
```

Then rebuild your system:
```bash
sudo nixos-rebuild switch --flake .
```

### Home Manager Installation

Add to your home-manager configuration:

```nix
{
  inputs = {
    home-manager.url = "github:nix-community/home-manager";
    waycast.url = "git+https://gitgud.foo/thegrind/waycast";
  };

  outputs = { home-manager, waycast, ... }: {
    homeConfigurations.youruser = home-manager.lib.homeManagerConfiguration {
      modules = [
        waycast.homeManagerModules.waycast
        {
          programs.waycast.enable = true;
        }
      ];
    };
  };
}
```

Then apply the configuration:
```bash
home-manager switch --flake .
```

### Traditional Installation

If you're not using Nix, you can build from source:

#### Dependencies
- Qt6 (Core, Gui, Qml, Quick, QuickControls2, Widgets, Wayland)
- Layer Shell Qt
- GLib/GIO
- CMake 3.21+
- Ninja (recommended)
- C++20 compiler

#### Build
```bash
git clone https://gitgud.foo/thegrind/waycast
cd waycast
make configure  # or: cmake -S . -B build -G Ninja
make bld        # or: cmake --build build
make install    # or: cp ./build/waycast ~/bin/waycast
```

## Usage

Launch waycast and start typing:

- **Applications**: Type app name (e.g., `firefox`, `kate`)
- **Files**: Type filename (e.g., `config.txt`)
- **Path search**: Type path + filename (e.g., `wallpapers/sunset`)
- **Directory browse**: Type path ending with `/` (e.g., `Documents/`)

### Keyboard Shortcuts

- `Escape` - Close launcher
- `↑/↓` - Navigate results  
- `Enter` - Execute selected item

## Development

### Development Shell

```bash
nix develop git+https://gitgud.foo/thegrind/waycast
```

Or locally:
```bash
nix develop
```

Inside the dev shell:
```bash
make configure  # Configure build
make bld       # Build project
make run       # Run waycast
make br        # Build and run
```

### Architecture

- **Plugin system**: `lib/plugins/` - Extensible search providers
- **Fuzzy search**: `lib/fuzzy.hpp` - AVX2-accelerated string matching
- **UI**: Qt6 QML with Material Dark theme
- **File utilities**: `lib/files.hpp` - File system operations
- **Icon resolution**: `lib/IconUtil.hpp` - System theme integration

### Build Systems

WayCast supports multiple build systems:

- **CMake + Ninja** (recommended): `make configure && make bld`
- **Zig**: `zig build run`
- **Nix**: `nix build`

## Configuration

Currently, WayCast works out of the box with sensible defaults:

- **Search directories**: `~/Documents`, `~/Desktop`, `~/Downloads`
- **Search depth**: 3 levels
- **File limit**: 1000 files
- **Ignored directories**: `node_modules`, `.git`, `build`, etc.

## Requirements

- **Linux** with Wayland compositor (Sway, Hyprland, KDE Plasma, GNOME, etc.)
- **Qt6** with Wayland support
- **Layer Shell Qt** for proper window positioning

## License

MIT

## Contributing

1. Fork the repository
2. Create your feature branch
3. Make your changes
4. Test with `nix build` and `nix run`
5. Submit a pull request

The development environment includes all dependencies and development tools needed to contribute.