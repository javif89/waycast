{
  description = "Waycast - application launcher for Wayland compositors";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      inherit (nixpkgs) lib;

      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = lib.genAttrs systems;

      cargoManifest = builtins.fromTOML (builtins.readFile ./Cargo.toml);

      pkgsFor =
        system:
        import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

      mkWaycastPackage =
        pkgs:
        let
          rustToolchain = pkgs.rust-bin.stable."1.94.0".default;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };

          buildLibraries = with pkgs; [
            expat
            fontconfig
            freetype
            libGL
            libx11
            libxcursor
            libxi
            libxrandr
            wayland
            libxkbcommon
            vulkan-loader
            glib
          ];

          # Do not add vulkan-loader here. Forcing Nix's pinned loader into
          # LD_LIBRARY_PATH makes Iced/WGPU render a black window on some systems.
          # Keep it in buildLibraries and let the host graphics stack select the
          # runtime Vulkan loader.
          runtimeLibraries = with pkgs; [
            libGL
            libx11
            libxcursor
            libxi
            libxinerama
            libxrandr
            wayland
            libxkbcommon
            glib
          ];
        in
        rustPlatform.buildRustPackage {
          pname = cargoManifest.package.name;
          version = cargoManifest.package.version;

          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./.sqlx
              ./Cargo.lock
              ./Cargo.toml
              ./assets
              ./migrations
              ./src
            ];
          };

          cargoLock.lockFile = ./Cargo.lock;
          SQLX_OFFLINE = "true";
          doCheck = true;

          nativeBuildInputs = with pkgs; [
            pkg-config
            makeWrapper
          ];

          buildInputs = buildLibraries;

          postInstall = ''
            mkdir -p "$out/share/waycast/icons"
            cp -r assets/icons/. "$out/share/waycast/icons/"
          '';

          preFixup = ''
            wrapProgram "$out/bin/waycast" \
              --prefix XDG_DATA_DIRS : "${pkgs.hicolor-icon-theme}/share:${pkgs.adwaita-icon-theme}/share" \
              --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath runtimeLibraries}"
          '';

          meta = {
            description = "Iced-based application launcher for Wayland compositors";
            homepage = "https://waycast.dev";
            downloadPage = "https://gitgud.boo/javif89/waycast";
            license = lib.licenses.mit;
            maintainers = [
              {
                name = "Javier Feliz";
                email = "me@javierfeliz.com";
              }
            ];
            mainProgram = "waycast";
            platforms = lib.platforms.linux;
          };
        };
    in
    {
      packages = forAllSystems (
        system:
        let
          waycast = mkWaycastPackage (pkgsFor system);
        in
        {
          inherit waycast;
          default = waycast;
        }
      );

      formatter = forAllSystems (system: (pkgsFor system).nixfmt-rfc-style);

      homeManagerModules.default =
        { lib, pkgs, ... }:
        {
          imports = [ ./modules/home-manager/waycast.nix ];
          programs.waycast.package = lib.mkDefault self.packages.${pkgs.stdenv.hostPlatform.system}.waycast;
        };

      homeModules.default = self.homeManagerModules.default;
    };
}
