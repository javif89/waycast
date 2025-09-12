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
          version = "0.0.2";
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
            wrapGAppsHook4
          ];

          buildInputs = with pkgs; [
            # GTK4 stack
            gtk4
            glib
            gdk-pixbuf
            pango
            cairo
            harfbuzz
            librsvg

            # Wayland + layer shell (GTK4 variant)
            wayland
            gtk4-layer-shell
          ];

          # Install custom icons
          postInstall = ''
            mkdir -p $out/share/waycast/icons
            cp -r assets/icons/* $out/share/waycast/icons/
          '';

          # wrapGAppsHook4 handles most GTK runtime setup automatically
          # Just ensure icon themes are available
          preFixup = ''
            gappsWrapperArgs+=(
              --prefix XDG_DATA_DIRS : "${pkgs.hicolor-icon-theme}/share:${pkgs.adwaita-icon-theme}/share"
            )
          '';

          meta = with pkgs.lib; {
            description = "GTK4-based application launcher for Wayland compositors";
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

            # GTK4 stack
            gtk4
            glib
            gdk-pixbuf
            pango
            cairo
            harfbuzz
            librsvg

            # Wayland + layer shell (GTK4 variant)
            wayland
            gtk4-layer-shell

            # Icons (so themed icons resolve)
            hicolor-icon-theme
            adwaita-icon-theme
          ];
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
