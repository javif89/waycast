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
    (flake-utils.lib.eachDefaultSystem (
      system:
      let
        # pkgs = nixpkgs.legacyPackages.${system};
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "waycast";
          version = "0.3.0";
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
            # Iced dependencies
            libGL
            wayland
            libxkbcommon
            xorg.libXcursor
            xorg.libX11
            xorg.libXrandr
            xorg.libXi
            vulkan-loader

            # For icons (still needed for desktop entries)
            glib
            pango
            cairo
            gdk-pixbuf
            harfbuzz
          ];

          # Install custom icons and patch binary
          postInstall = ''
            mkdir -p $out/share/waycast/icons
            cp -r assets/icons/* $out/share/waycast/icons/

            # Patch the binary with required library paths for Iced
            patchelf --set-rpath "${pkgs.lib.makeLibraryPath [
              pkgs.libGL
              pkgs.wayland
              pkgs.libxkbcommon
              pkgs.xorg.libXcursor
              pkgs.xorg.libX11
              pkgs.xorg.libXrandr
              pkgs.xorg.libXi
              pkgs.vulkan-loader
              pkgs.glib
              pkgs.pango
              pkgs.cairo
              pkgs.gdk-pixbuf
              pkgs.harfbuzz
            ]}:$out/lib" $out/bin/waycast
          '';

          # Wrap the binary with necessary environment variables
          preFixup = ''
            wrapProgram $out/bin/waycast \
              --prefix XDG_DATA_DIRS : "${pkgs.hicolor-icon-theme}/share:${pkgs.adwaita-icon-theme}/share" \
              --set LD_LIBRARY_PATH "${pkgs.lib.makeLibraryPath [
                pkgs.libGL
                pkgs.wayland
                pkgs.libxkbcommon
                pkgs.xorg.libXcursor
                pkgs.vulkan-loader
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

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Build tools
            pkg-config
            oranda
            patchelf
            cmake
            clang

            # Iced dependencies
            libGL
            wayland
            libxkbcommon
            xorg.libXcursor
            xorg.libX11
            xorg.libXrandr
            xorg.libXi
            vulkan-loader
            vulkan-headers
            vulkan-validation-layers

            # For icon support (still needed for desktop entries)
            glib
            pango
            cairo
            gdk-pixbuf
            harfbuzz

            # Icons (so themed icons resolve)
            hicolor-icon-theme
            adwaita-icon-theme

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
        waycast = self.packages.${final.stdenv.hostPlatform.system}.default;
      };

      homeManagerModules.default = import ./modules/home-manager/waycast.nix;
    };
}
