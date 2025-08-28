{
  pkgs ? import <nixpkgs> { },
}:
let
  qt = pkgs.qt6Packages;
in
pkgs.mkShell {
  buildInputs = [
    pkgs.cmake
    pkgs.ninja
    pkgs.clang_18
    pkgs.lldb
    pkgs.ccacheWrapper
    pkgs.glib
    pkgs.glib.dev
    qt.full
    qt.qtbase
    qt.qtdeclarative # <- provides QML/Quick + Quick Controls 2 (QtQuick.Controls)
    qt.qtwayland # <- provides the Wayland platform plugin
    qt.qt5compat # <- Required for some theming features
    pkgs.kdePackages.layer-shell-qt # <- provides layer shell support
    pkgs.kdePackages.layer-shell-qt.dev # <- provides headers
    pkgs.pkg-config

    # THEMING - Qt6 specific
    pkgs.qt6ct
    pkgs.kdePackages.qqc2-desktop-style
    pkgs.kdePackages.breeze-icons
    pkgs.hicolor-icon-theme
    pkgs.papirus-icon-theme
    pkgs.kdePackages.breeze  # Full Breeze theme
    qt.qtsvg  # Required for theme icons
  ];

  shellHook = ''
    export QT_PLUGIN_PATH='${qt.full}'
    export QML2_IMPORT_PATH='${qt.full}:${pkgs.kdePackages.layer-shell-qt}/lib/qt-6/qml'
    export QT_QPA_PLATFORM=wayland
    
    # Force Qt6 theming - override any Qt5 settings from Stylix  
    unset QT_QPA_PLATFORMTHEME  # Clear system setting first
    unset QT_STYLE_OVERRIDE     # Clear style overrides
    export QT_QPA_PLATFORMTHEME=qt6ct
    
    # For QML applications, we might need different approach
    export QT_QUICK_CONTROLS_STYLE=Material
    export QT_QUICK_CONTROLS_MATERIAL_THEME=Dark
    export QT_QUICK_CONTROLS_MATERIAL_VARIANT=Dense
    
    # Icon theme - prefer Papirus if available, fallback to Breeze
    export QT_ICON_THEME=Papirus
    
    # Create qt6ct configuration for Gruvbox Dark theme
    mkdir -p ~/.config/qt6ct
    cat > ~/.config/qt6ct/qt6ct.conf << 'EOF'
[Appearance]
style=Fusion
color_scheme_path=
custom_palette=true
standard_dialogs=default
icon_theme=Papirus-Dark

[Fonts]
fixed="Monospace,9,-1,5,400,0,0,0,0,0,0,0,0,0,0,1"
general="Sans Serif,9,-1,5,400,0,0,0,0,0,0,0,0,0,0,1"

[Interface]
activate_item_on_single_click=1
buttonbox_layout=0
cursor_flash_time=1000
dialog_buttons_have_icons=1
double_click_interval=400
gui_effects=@Invalid()
keyboard_scheme=2
menus_have_icons=true
show_shortcuts_in_context_menus=true
stylesheets=@Invalid()
toolbutton_style=4
underline_shortcut=1
wheel_scroll_lines=3

[PaletteEditor]
geometry=@ByteArray(\x1\xd9\xd0\xcb\0\x3\0\0\0\0\0\0\0\0\0\0\0\0\x2V\0\0\x1\x14\0\0\0\0\0\0\0\0\0\0\x2V\0\0\x1\x14\0\0\0\0\x2\0\0\0\a\x80\0\0\0\0\0\0\0\0\0\0\x2V\0\0\x1\x14)

[QSSEditor]
geometry=@ByteArray(\x1\xd9\xd0\xcb\0\x3\0\0\0\0\0\0\0\0\0\0\0\0\x3 \0\0\x2X\0\0\0\0\0\0\0\0\0\0\x3 \0\0\x2X\0\0\0\0\x2\0\0\0\a\x80\0\0\0\0\0\0\0\0\0\0\x3 \0\0\x2X)

[SettingsWindow]
geometry=@ByteArray(\x1\xd9\xd0\xcb\0\x3\0\0\0\0\0\0\0\0\0\0\0\0\x2\x84\0\0\x1\xe0\0\0\0\0\0\0\0\0\0\0\x2\x84\0\0\x1\xe0\0\0\0\0\x2\0\0\0\a\x80\0\0\0\0\0\0\0\0\0\0\x2\x84\0\0\x1\xe0)

[Troubleshooting]
force_raster_widgets=1
ignored_applications=@Invalid()
EOF

    # Create Gruvbox Dark color scheme
    cat > ~/.config/qt6ct/colors/GruvboxDark.conf << 'EOF'
[ColorScheme]
active_colors=#ffebdbb2, #ff3c3836, #ff504945, #ff32302f, #ff1d2021, #ff282828, #ffebdbb2, #fffbf1c7, #ffebdbb2, #ff282828, #ff1d2021, #ff0d1011, #ff458588, #ff282828, #ff83a598, #ffcc241d, #ff32302f, #ff000000, #fffdf4c1, #ffebdbb2, #ff928374
disabled_colors=#ff928374, #ff3c3836, #ff504945, #ff32302f, #ff1d2021, #ff282828, #ff928374, #fffbf1c7, #ff928374, #ff282828, #ff1d2021, #ff0d1011, #ff32302f, #ff928374, #ff458588, #ffcc241d, #ff32302f, #ff000000, #fffdf4c1, #ffebdbb2, #ff928374
inactive_colors=#ffebdbb2, #ff3c3836, #ff504945, #ff32302f, #ff1d2021, #ff282828, #ffebdbb2, #fffbf1c7, #ffebdbb2, #ff282828, #ff1d2021, #ff0d1011, #ff32302f, #ff282828, #ff83a598, #ffcc241d, #ff32302f, #ff000000, #fffdf4c1, #ffebdbb2, #ff928374
EOF

    # Update qt6ct.conf to use the custom color scheme
    sed -i 's|color_scheme_path=|color_scheme_path=~/.config/qt6ct/colors/GruvboxDark.conf|' ~/.config/qt6ct/qt6ct.conf
    
    echo "--------------------------------------------"
    echo "Qt6 Gruvbox Dark theme configured!"
    echo "QT_PLUGIN_PATH: $QT_PLUGIN_PATH"
    echo "QT_QPA_PLATFORMTHEME: $QT_QPA_PLATFORMTHEME"
    echo "QT_ICON_THEME: $QT_ICON_THEME"
  '';
}
