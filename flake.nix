{
  description = "WayCast - A Raycast-like application launcher for Linux with Qt6 and Wayland support";

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
        pkgs = nixpkgs.legacyPackages.${system};

        waycast = pkgs.stdenv.mkDerivation rec {
          pname = "waycast";
          version = "0.0.1";

          src = ./.;

          nativeBuildInputs = with pkgs; [
            cmake
            ninja
            pkg-config
            qt6.wrapQtAppsHook
          ];

          buildInputs = with pkgs; [
            # Qt6 components
            qt6.qtbase
            qt6.qtquick3d
            qt6.qtdeclarative # For QML/Quick
            qt6.qtwayland

            # Layer Shell Qt for Wayland
            layer-shell-qt

            # GLib/GIO dependencies
            glib

            # Other system dependencies
            wayland
            wayland-protocols
          ];

          # Set Qt6 module path and other environment variables
          qtWrapperArgs = [
            "--prefix QML2_IMPORT_PATH : ${qt6.qtdeclarative}/${qt6.qtbase.qtQmlPrefix}"
            "--prefix QT_PLUGIN_PATH : ${qt6.qtbase.qtPluginPrefix}"
          ];

          cmakeFlags = [
            "-GNinja"
            "-DCMAKE_BUILD_TYPE=Release"
          ];

          # Enable Qt6 and Wayland features
          postPatch = ''
            # Ensure we can find Qt6 components
            substituteInPlace CMakeLists.txt \
              --replace 'find_package(Qt6' 'find_package(Qt6' \
              --replace 'find_package(LayerShellQt REQUIRED)' 'find_package(LayerShellQt REQUIRED)'
          '';

          meta = with pkgs.lib; {
            description = "A Raycast-like application launcher for Linux with Qt6 and Wayland support";
            homepage = "https://gitgud.foo/thegrind/waycast";
            license = licenses.mit; # Adjust if different license
            maintainers = [ "thegrind" ];
            platforms = platforms.linux;
            mainProgram = "waycast";
          };
        };

      in
      {
        # Default package
        packages.default = waycast;
        packages.waycast = waycast;

        # Development shell with all dependencies
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Build tools
            cmake
            ninja
            pkg-config
            clang
            gdb

            # Qt6 development
            qt6.qtbase
            qt6.qtquick3d
            qt6.qtdeclarative
            qt6.qtwayland
            qt6.qttools # For Qt development tools

            # Layer Shell Qt
            layer-shell-qt

            # GLib/GIO
            glib.dev

            # Wayland
            wayland
            wayland-protocols

            # Optional development tools
            qtcreator # Qt IDE
            valgrind # Memory debugging
          ];

          shellHook = ''
            echo "WayCast development environment"
            echo "Available commands:"
            echo "  make configure - Configure build with CMake"
            echo "  make bld       - Build the project"
            echo "  make run       - Run waycast"
            echo "  make install   - Install to ~/bin/"
            echo ""
            echo "Qt6 and Wayland development tools are available"

            # Set up Qt6 environment
            export QT_QPA_PLATFORM=wayland
            export QML2_IMPORT_PATH="${pkgs.qt6.qtdeclarative}/${pkgs.qt6.qtbase.qtQmlPrefix}:$QML2_IMPORT_PATH"
          '';
        };

        # Apps for nix run
        apps.default = flake-utils.lib.mkApp {
          drv = waycast;
          name = "waycast";
        };

        # Allow building on other architectures
        hydraJobs = {
          build = waycast;
        };
      }
    )
    // {
      # NixOS module for system-wide installation
      nixosModules.waycast =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        with lib;
        let
          cfg = config.programs.waycast;
        in
        {
          options.programs.waycast = {
            enable = mkEnableOption "WayCast application launcher";

            package = mkOption {
              type = types.package;
              default = self.packages.${pkgs.system}.default;
              description = "WayCast package to use";
            };
          };

          config = mkIf cfg.enable {
            environment.systemPackages = [ cfg.package ];

            # Ensure Wayland support
            programs.wayland.enable = mkDefault true;

            # Install desktop entry (if you create one)
            # environment.etc."applications/waycast.desktop" = {
            #   text = ''
            #     [Desktop Entry]
            #     Name=WayCast
            #     Comment=Application launcher for Wayland
            #     Exec=${cfg.package}/bin/waycast
            #     Type=Application
            #     Categories=Utility;
            #   '';
            # };
          };
        };

      # Home Manager module
      homeManagerModules.waycast =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        with lib;
        let
          cfg = config.programs.waycast;
        in
        {
          options.programs.waycast = {
            enable = mkEnableOption "WayCast application launcher";

            package = mkOption {
              type = types.package;
              default = self.packages.${pkgs.system}.default;
              description = "WayCast package to use";
            };

            settings = mkOption {
              type = types.attrs;
              default = { };
              description = "WayCast configuration";
            };
          };

          config = mkIf cfg.enable {
            home.packages = [ cfg.package ];

            # You can add configuration file generation here if needed
            # xdg.configFile."waycast/config.json" = mkIf (cfg.settings != {}) {
            #   text = builtins.toJSON cfg.settings;
            # };
          };
        };
    };
}
