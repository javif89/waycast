{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

let
  icedGUILibs = with pkgs; [
    libGL
    libX11
    libXcursor
    libXi
    libXrandr
    wayland
    libxkbcommon
  ];
in
{
  dotenv.disableHint = true;

  packages =
    with pkgs;
    [
      # For editing nix files
      nixd

      # Build tools
      pkg-config
      patchelf
      clang
      cargo-dist
      just

      # Iced dependencies (from official docs)
      expat
      fontconfig
      freetype
      freetype.dev
      vulkan-loader
      vulkan-headers
      vulkan-validation-layers

      # Used by the daemon and launcher modules
      glib

      # CLI Utils
      sqlx-cli

      # Utilities
      socat

      # Cargo tools
      cargo-machete

      # Icons (so themed icons resolve)
      hicolor-icon-theme

      # Benchmarking / Profiling
      linuxKernel.packages.linux_6_6.perf
      valgrind
      heaptrack
      gdb
    ]
    ++ icedGUILibs;

  languages = {
    rust = {
      enable = true;
    };
  };

  env = {
    # TODO: Refactor to do:
    # RUSTFLAGS = "-C link-arg=-Wl,-rpath,${lib.makeLibraryPath dlopenLibs}";
    # Instead of just setting a global library path
    LD_LIBRARY_PATH = lib.makeLibraryPath icedGUILibs;
    LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
  };

  scripts = {
    "run".exec = ''
      cargo build --release
      ./target/release/waycast $1
    '';

    "profile:heaptrack".exec = ''
      cargo build -p waycast
      heaptrack -- ./target/debug/waycast
    '';

    "dev:reset-db".exec = ''
      rm xdg/waycast.db -f
      rm xdg/waycast.db-shm -f
      rm xdg/waycast.db-wal -f
      touch xdg/waycast.db
      sqlx migrate run --source ./migrations --database-url sqlite://xdg/waycast.db
    '';

    "dev:prepare-sqlx".exec = ''
      set -euo pipefail

      SQLX_TMP_DIR="$(mktemp -d)"
      trap 'rm -rf "$SQLX_TMP_DIR"' EXIT

      SQLX_DATABASE_URL="sqlite://$SQLX_TMP_DIR/waycast.db"
      sqlx database create --database-url "$SQLX_DATABASE_URL"
      sqlx migrate run --source ./migrations --database-url "$SQLX_DATABASE_URL"
      cargo sqlx prepare \
        --no-dotenv \
        --database-url "$SQLX_DATABASE_URL" \
        -- \
        --all-targets
    '';

    "dev:make-devicon-theme".exec = ''
      DEVICON_DIR="./assets/icons/devicons"

      rm -rf $DEVICON_DIR
      mkdir -p $DEVICON_DIR 
      devicon remix -t framework,language -o $DEVICON_DIR --variant original --fallback plain
      devicon get nixos -o $DEVICON_DIR
      devicon get bash -o $DEVICON_DIR
      devicon get ansible -o $DEVICON_DIR
      cp $DEVICON_DIR/nixos.svg $(DEVICON_DIR)/nix.svg
      cp $DEVICON_DIR/bash.svg $(DEVICON_DIR)/shell.svg
    '';

    # TODO: Fix this
    "dev:install-devicons".exec = ''
      @XDG_DATA_HOME=$${XDG_DATA_HOME:-$$HOME/.local/share} && \
      ICON_DIR="$$XDG_DATA_HOME/waycast/icons" && \
      mkdir -p "$$ICON_DIR" && \
      cp -r ./assets/icons/* "$$ICON_DIR/" && \
      echo "Icons installed to $$ICON_DIR"
    '';

    "release:tag".exec = ''
      echo "Tagging release $1"
      git add -A
      git commit -m "chore(release): v$1"
      git tag -fa v$1 -m "Release v$1"
      echo "Release v$1 created!"
    '';

    "release:push".exec = ''
      git push origin master
      git push --force origin v$1
      @echo "Release pushed to origin"
    '';

    "release".exec = ''
      release:tag $1
      release:push $1
    '';
  };

  enterShell = ''
    echo "WAYLAND_DISPLAY: $WAYLAND_DISPLAY"
    echo "DISPLAY: $DISPLAY"
    echo "XDG_SESSION_TYPE: $XDG_SESSION_TYPE"
  '';

  # NOTE: This is from the beginnings of waycast when I was using GTK.
  # Shouldn't be needed anymore, but if there's ever issues with
  # the UI try uncommenthing this.
  # enterShell = ''
  #   export GDK_BACKEND=wayland,x11
  #   export QT_QPA_PLATFORM=wayland;xcb
  #   export SDL_VIDEODRIVER=wayland
  #   export CLUTTER_BACKEND=wayland

  #   # Inherit display variables from parent environment if available
  #   if [ -n "$WAYLAND_DISPLAY" ]; then
  #     export WAYLAND_DISPLAY="$WAYLAND_DISPLAY"
  #   fi
  #   if [ -n "$DISPLAY" ]; then
  #     export DISPLAY="$DISPLAY"
  #   fi
  #   if [ -n "$XDG_RUNTIME_DIR" ]; then
  #     export XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR"
  #   fi

  #   # Set XDG_SESSION_TYPE if not already set
  #   if [ -z "$XDG_SESSION_TYPE" ]; then
  #     if [ -n "$WAYLAND_DISPLAY" ]; then
  #       export XDG_SESSION_TYPE=wayland
  #     elif [ -n "$DISPLAY" ]; then
  #       export XDG_SESSION_TYPE=x11
  #     fi
  #   fi

  # echo "Display environment setup complete"
  # echo "WAYLAND_DISPLAY: $WAYLAND_DISPLAY"
  # echo "DISPLAY: $DISPLAY"
  # echo "XDG_SESSION_TYPE: $XDG_SESSION_TYPE"
  # '';
}
