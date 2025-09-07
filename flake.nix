{
  description = "Waycast - application launcher for Wayland compositors";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        # pkgs = nixpkgs.legacyPackages.${system};
        pkgs = import nixpkgs {
          inherit system;
          overlays = [self.overlay];
        };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "waycast";
          version = "0.0.1";
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

          # Wrap binary to ensure icon themes are available
          postInstall = ''
            wrapProgram $out/bin/waycast \
              --prefix XDG_DATA_DIRS : "${pkgs.hicolor-icon-theme}/share:${pkgs.adwaita-icon-theme}/share"
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
      }
    );

    overlay = final: prev: {
      waycast = self.packages.${final.system}.default;
    };

    homeManagerModules.waycast = import ./modules/home-manager/waycast.nix
}
