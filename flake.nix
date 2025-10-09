{
  description = "Waycast - application launcher for Wayland compositors";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    home-manager.url = "github:nix-community/home-manager";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      home-manager,
    }:
    let
      # Define the package as a function that takes pkgs as an argument
      # This allows it to be built with any nixpkgs instance
      mkWaycastPackage = pkgs: pkgs.rustPlatform.buildRustPackage {
        pname = "waycast";
        version = "0.4.0";
        src = self;

        cargoLock.lockFile = ./Cargo.lock;

        doCheck = true;
        cargoTestFlags = [
          "--bins"
          "--tests"
        ];

        nativeBuildInputs = with pkgs; [
          pkg-config
          makeWrapper
          patchelf
        ];

        buildInputs = with pkgs; [
          # Iced dependencies (from official docs)
          expat
          fontconfig
          freetype
          libGL
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          wayland
          libxkbcommon
          vulkan-loader

          # Still needed by waycast-plugins for file icons and launching
          glib
        ];

        # These will be automatically available when waycast is installed
        propagatedBuildInputs = with pkgs; [
          libGL
          vulkan-loader
          wayland
          libxkbcommon
          glib
        ];

        # Install custom icons
        postInstall = ''
          mkdir -p $out/share/waycast/icons
          cp -r assets/icons/* $out/share/waycast/icons/
          # Don't patch RPATH - the binary already works with Cargo's RPATH
        '';

        # Wrap the binary with necessary environment variables (same libs as devShell)
        preFixup = ''
          wrapProgram $out/bin/waycast \
            --prefix XDG_DATA_DIRS : "${pkgs.hicolor-icon-theme}/share:${pkgs.adwaita-icon-theme}/share" \
            --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath [
              pkgs.libGL
              pkgs.xorg.libXrandr
              pkgs.xorg.libXinerama
              pkgs.xorg.libXcursor
              pkgs.xorg.libXi
              pkgs.wayland
              pkgs.libxkbcommon
              pkgs.glib
            ]}"
        '';

        meta = with pkgs.lib; {
          description = "Iced-based application launcher for Wayland compositors";
          homepage = "https://gitgud.foo/thegrind/waycast";
          license = licenses.mit;
          maintainers = [ "Javier Feliz" ];
          platforms = platforms.linux;
        };
      };
    in
    (flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        # Build the package with the flake's own nixpkgs for direct installation
        packages.default = mkWaycastPackage pkgs;

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Build tools
            pkg-config
            patchelf
            cmake
            clang

            # Iced dependencies (from official docs)
            expat
            fontconfig
            freetype
            freetype.dev
            libGL
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
            wayland
            libxkbcommon
            vulkan-loader
            vulkan-headers
            vulkan-validation-layers

            # Still needed by waycast-plugins
            glib

            # Icons (so themed icons resolve)
            hicolor-icon-theme

            # Benchmarking / Profiling
            linuxKernel.packages.linux_6_6.perf
            valgrind
          ];

          LD_LIBRARY_PATH =
            with pkgs;
            lib.makeLibraryPath [
              libGL
              xorg.libXrandr
              xorg.libXinerama
              xorg.libXcursor
              xorg.libXi
              wayland
              libxkbcommon
            ];
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

          # Ensure display environment variables are available
          shellHook = ''
            export GDK_BACKEND=wayland,x11
            export QT_QPA_PLATFORM=wayland;xcb
            export SDL_VIDEODRIVER=wayland
            export CLUTTER_BACKEND=wayland

            # Inherit display variables from parent environment if available
            if [ -n "$WAYLAND_DISPLAY" ]; then
              export WAYLAND_DISPLAY="$WAYLAND_DISPLAY"
            fi
            if [ -n "$DISPLAY" ]; then
              export DISPLAY="$DISPLAY"
            fi
            if [ -n "$XDG_RUNTIME_DIR" ]; then
              export XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR"
            fi

            # Set XDG_SESSION_TYPE if not already set
            if [ -z "$XDG_SESSION_TYPE" ]; then
              if [ -n "$WAYLAND_DISPLAY" ]; then
                export XDG_SESSION_TYPE=wayland
              elif [ -n "$DISPLAY" ]; then
                export XDG_SESSION_TYPE=x11
              fi
            fi

            echo "Display environment setup complete"
            echo "WAYLAND_DISPLAY: $WAYLAND_DISPLAY"
            echo "DISPLAY: $DISPLAY"
            echo "XDG_SESSION_TYPE: $XDG_SESSION_TYPE"
          '';
        };

        # Move overlay outside system-specific outputs
      }
    ))
    // {
      overlays.default = final: prev: {
        # Build waycast using the consumer's nixpkgs (final)
        # This ensures all dependencies and wrappers use the correct store paths
        waycast = mkWaycastPackage final;
      };

      homeManagerModules.default = import ./modules/home-manager/waycast.nix;
    };
}
