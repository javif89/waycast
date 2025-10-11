# Waycast

A launcher for Wayland that doesn't suck. Think Raycast but for Linux.

## Special Thanks

- [DevIcon](https://devicon.dev/). Used for the project folder icons
- [Nucleo Matcher](https://github.com/helix-editor/nucleo). Amazing fuzzy finder library. It powers the search functionality in all the plugins. Very fast.

## Tools I've made as part of developing waycast

Who knows how many it will end up being. So I'll be keeping track below.

- [FreeDesktop](https://github.com/javif89/freedesktop). A rust implementation of the freedesktop spec (in progress). I was relying on GIO for these kinds of things, but the library is very annoying to use and some times not reliable.
- [Devicon CLI](https://gitgud.foo/javif89/devicon-cli). Needed an easy way to manage and remix the devicon set for this project.
- [mathengine](https://github.com/javif89/mathengine). For the calculator functionality

## What is this?

Waycast is an application launcher built for Wayland desktops. It's fast, extensible, and designed to get out of your way while helping you find what you need.

**Current features:**
- Search and launch desktop applications
- Search files in your home directories (Documents, Pictures, Music, Videos)
- Fuzzy search that actually works
- Fast startup with background file indexing
- Iced UI with proper layer shell integration

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
- **waycast-ui** - Iced UI and main binary

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
└── waycast-ui/            # Iced UI (main app)
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

### Dependencies

WayCast's dependencies are primarily based on [Iced's dependencies](https://github.com/iced-rs/iced/blob/master/DEPENDENCIES.md),
with additional requirements for the plugin system. For development and nix installations, the flake.nix takes care of these.
If you're installing WayCast on a different system, make sure you have the following libraries available:

**Iced requirements:**
- expat
- fontconfig
- freetype
- freetype.dev
- libGL
- pkg-config
- xorg.libX11
- xorg.libXcursor
- xorg.libXi
- xorg.libXrandr
- wayland
- libxkbcommon

**Plugin requirements:**
- glib (for file type detection and launching)

### IMPORTANT NOTE FOR HYPRLAND

If you want waycast to start up even faster, remove the default "fade" animation hyprland adds to layer shell windows:

**Nix**

```nix
layerrule = [
  "noanim, Waycast"
];
```

**Other**

```
layerrule = "noanim, Waycast";
```

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