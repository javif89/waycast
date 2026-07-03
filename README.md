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

This is a single Cargo package with three main modules:

- **core** - Domain models, search, configuration, persistence, and item launching
- **daemon** - Application, file, and project scanners plus filesystem watching
- **ui** - The Iced layer-shell interface

### Testing the nix flake packaging

`nix build`
`nix run`

## Packaging/release process

1. `dev:prepare-sqlx` to generate the necessary sqlx data

### Project Structure

```
waycast/
├── migrations/             # SQLite migrations
└── src/
    ├── core/               # Models, search, config, data, and launching
    ├── daemon/             # Scanning, indexing, and filesystem watching
    ├── ui/                 # Iced UI
    └── main.rs             # CLI and process orchestration
```

The core module is UI-agnostic. The daemon depends on core, the UI consumes core services, and `main.rs` composes the daemon and UI into the `waycast` executable.

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

### Nix installation

Waycast is not currently packaged in nixpkgs. Add this repository as a flake
input:

```nix
inputs.waycast.url = "git+https://gitgud.boo/javif89/waycast";
```

#### Home Manager (recommended)

Import the Waycast module in your Home Manager configuration. No nixpkgs
overlay is required.

```nix
{
  imports = [ inputs.waycast.homeModules.default ];

  programs.waycast = {
    enable = true;

    settings = {
      plugins.projects = {
        search_paths = [ "/absolute/path/to/projects" ];
        skip_dirs = [ "node_modules" "target" ".git" ];
        open_command = "code -n {path}";
      };

      plugins.file_search = {
        search_paths = [ "/absolute/path/to/search" ];
        ignore_dirs = [ "scripts" "temp" ];
      };
    };
  };
}
```

For Home Manager configured as a NixOS module, place the import and
`programs.waycast` configuration under your user:

```nix
home-manager.users.youruser = {
  imports = [ inputs.waycast.homeModules.default ];
  programs.waycast.enable = true;
};
```

For standalone Home Manager, add the module to `homeConfigurations`:

```nix
homeConfigurations.youruser = home-manager.lib.homeManagerConfiguration {
  pkgs = nixpkgs.legacyPackages.x86_64-linux;
  modules = [
    waycast.homeModules.default
    {
      programs.waycast.enable = true;
    }
  ];
};
```

Enabling the module installs Waycast and creates `waycast-daemon.service`, a
systemd user service that starts with `graphical-session.target` and restarts
the background process if it exits. Running `waycast` from a terminal or window
manager keybind then signals the background process to open the UI.

Your desktop environment or window manager must activate the systemd user
graphical session. For example, Home Manager's Hyprland module should have its
systemd integration enabled. After applying your configuration, check the
service with:

```bash
systemctl --user status waycast-daemon.service
```

#### Package only

The package is also available without the Home Manager module:

```nix
# NixOS
environment.systemPackages = [
  inputs.waycast.packages.${pkgs.stdenv.hostPlatform.system}.default
];

# Home Manager
home.packages = [
  inputs.waycast.packages.${pkgs.stdenv.hostPlatform.system}.default
];
```

Or install it into a Nix profile:

```bash
nix profile install 'git+https://gitgud.boo/javif89/waycast'
```

Package-only installation does not create the required background service. You
must arrange for `waycast` to start with your graphical session yourself.

## Contributing

TBA

## License

MIT
