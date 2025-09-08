# Waycast

A launcher for Wayland that doesn't suck. Think Raycast but for Linux.

I already ordered the programmer socks.

## What is this?

Waycast is an application launcher built for Wayland desktops. It's fast, extensible, and designed to get out of your way while helping you find what you need.

**Current features:**
- Search and launch desktop applications
- Search files in your home directories (Documents, Pictures, Music, Videos)
- Fuzzy search that actually works
- Fast startup with background file indexing
- GTK4 with proper layer shell integration

**Planned features:**
- Background daemon for instant launches
- Plugin system for extensions
- Calculator, clipboard history, system controls
- Terminal UI for SSH sessions
- Web search integration

## Development

This is a Cargo workspace with three main crates:

- **waycast-core** - The launcher engine (traits, logic, no UI)
- **waycast-plugins** - Plugin implementations (desktop apps, file search)
- **waycast-gtk** - GTK4 UI and main binary

### Common Commands

```bash
make help           # See all available commands
make quick          # Format code + compile check
make test           # Run tests (that I don't have yet)
make build-all      # Build everything
make install        # Install to system
```

### Project Structure

```
waycast/
├── waycast-core/           # Core launcher logic
├── waycast-plugins/        # Plugin implementations
└── waycast-gtk/           # GTK UI (main app)
```

The core is deliberately minimal and UI-agnostic. Plugins depend on core. UI depends on both core and plugins. Nothing depends on the UI.

## Why Another Launcher?

Linux desktop launchers are either too basic (dmenu, wofi) or too bloated (some KDE thing with 47 configuration tabs). Raycast nailed the UX on macOS, but there's no good equivalent for Linux.

Waycast aims to be:
- **Fast** - Sub-100ms search responses, instant startup
- **Clean** - Good defaults, minimal configuration needed  
- **Extensible** - Plugin system for custom functionality
- **Native** - Proper Wayland integration, not an Electron app

## Installation

### Nix Flakes

Add to your `flake.nix` inputs:
```nix
waycast.url = "git+https://gitgud.foo/thegrind/waycast";
```

Add the overlay and Home Manager module:
```nix
nixpkgs.overlays = [ inputs.waycast.overlays.default ];

home-manager.users.youruser = {
  imports = [ inputs.waycast.homeManagerModules.default ];
  
  programs.waycast = {
    enable = true;
    settings = {
      plugins.projects = {
        search_paths = ["/absolute/path/to/search"];
        skip_dirs = [ "node_modules" "target" ".git" ];
        open_command = "code -n {path}";
      };
      plugins.file_search = {
        search_paths = ["/absolute/path/to/search"];
        ignore_dirs = ["scripts", "temp"];
      };
    };
    css = ''
      window {
        background: rgba(0, 0, 0, 0.8);
        border-radius: 12px;
      }
    '';
  };
};
```

**Just the package:**
```nix
nixpkgs.overlays = [ inputs.waycast.overlays.default ];
environment.systemPackages = [ pkgs.waycast ];
# or for home-manager:
home.packages = [ pkgs.waycast ];
```

## Contributing

TBA

## License

TBA