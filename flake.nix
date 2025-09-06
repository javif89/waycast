{
  description = "Waycast - GTK4-based application launcher for Wayland compositors";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
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
      });
}