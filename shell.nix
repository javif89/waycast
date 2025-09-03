# shell.nix
{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  # Tools you’ll use directly
  buildInputs = [
    pkgs.pkg-config

    # GTK4 stack
    pkgs.gtk4
    pkgs.glib
    pkgs.gdk-pixbuf
    pkgs.pango
    pkgs.cairo
    pkgs.harfbuzz

    # Wayland + layer shell (GTK4 variant)
    pkgs.wayland
    pkgs.gtk4-layer-shell

    # Icons (so themed icons resolve; harmless even if you don't use yet)
    pkgs.hicolor-icon-theme
    pkgs.adwaita-icon-theme
  ];

  shellHook = ''
    export GDK_BACKEND=wayland
    echo "gtk4: $(pkg-config --modversion gtk4 2>/dev/null || echo missing)"
    echo "gtk4-layer-shell: $(pkg-config --modversion gtk4-layer-shell-0 2>/dev/null || echo missing)"
    echo "wayland-client: $(pkg-config --modversion wayland-client 2>/dev/null || echo missing)"
    echo "XDG_SESSION_TYPE=$XDG_SESSION_TYPE"
    echo "WAYLAND_DISPLAY=$WAYLAND_DISPLAY"
    echo "GDK_BACKEND=$GDK_BACKEND"
  '';
}
