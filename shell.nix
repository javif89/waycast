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
    pkgs.kdePackages.layer-shell-qt # <- provides layer shell support
    pkgs.kdePackages.layer-shell-qt.dev # <- provides headers
    pkgs.pkg-config
    # qt.qttools
    # qt.qtshadertools
  ];

  shellHook = ''
    export QT_PLUGIN_PATH='${qt.full}'
    export QML2_IMPORT_PATH='${qt.full}:${pkgs.kdePackages.layer-shell-qt}/lib/qt-6/qml'
    export QT_QPA_PLATFORM=wayland
    echo "--------------------------------------------"
    echo "QT_PLUGIN_PATH: $QT_PLUGIN_PATH"
    echo "QML2_IMPORT_PATH: $QML2_IMPORT_PATH"
  '';
}
