{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.programs.waycast;

  # Convert a Nix attribute set to TOML format
  tomlFormat = pkgs.formats.toml { };
  toToml = value: tomlFormat.generate "waycast.toml" value;

in
{
  options.programs.waycast = {
    enable = mkEnableOption "waycast application launcher";

    package = mkOption {
      type = types.package;
      default = pkgs.waycast;
      description = "The waycast package to use";
    };

    enableDaemon = mkOption {
      type = types.bool;
      default = true;
      description = "Whether to enable the waycast daemon as a systemd user service";
    };

    settings = mkOption {
      type = types.attrs;
      default = { };
      example = literalExpression ''
        {
          plugins = {
            projects = {
              search_paths = [ "~/code" "~/projects" ];
              skip_dirs = [ "node_modules" "target" ".git" ];
              open_command = "code -n {path}";
            };
            file_search = {
              search_paths = [ "~/Documents" "~/Downloads" ];
              ignore_dirs = [ "cache" "vendor" ];
            };
          };
        }
      '';
      description = ''
        Waycast configuration. This will be converted to TOML format
        and placed in ~/.config/waycast/waycast.toml
      '';
    };

    css = mkOption {
      type = types.nullOr types.lines;
      default = null;
      example = ''
        window {
          background: rgba(0, 0, 0, 0.8);
          border-radius: 12px;
        }

        .search-entry {
          font-size: 16px;
          padding: 12px;
        }
      '';
      description = ''
        Custom GTK CSS styling for waycast.
        This will be placed in ~/.config/waycast/waycast.css
      '';
    };
  };

  config = mkIf cfg.enable {
    home.packages = [ cfg.package ];

    xdg.configFile = mkMerge [
      (mkIf (cfg.settings != { }) {
        "waycast/waycast.toml".source = toToml cfg.settings;
      })

      (mkIf (cfg.css != null) {
        "waycast/waycast.css".text = cfg.css;
      })
    ];

    # Ensure cache and data dirs exist to avoid runtime errors in the future
    home.file."${config.xdg.cacheHome}/waycast/.keep".text = "";
    home.file."${config.xdg.dataHome}/waycast/.keep".text = "";

    # Install waycast icons to XDG_DATA_HOME
    home.file."${config.xdg.dataHome}/waycast/icons" = {
      source = "${cfg.package}/share/waycast/icons";
      recursive = true;
    };

    # Enable waycast daemon as systemd user service
    systemd.user.services.waycast-daemon = mkIf cfg.enableDaemon {
      Unit = {
        Description = "Waycast application launcher daemon";
        Documentation = "https://waycast.dev";
        After = [ "graphical-session.target" ];
        PartOf = [ "graphical-session.target" ];
      };

      Service = {
        Type = "simple";
        ExecStart = "${cfg.package}/bin/waycast-daemon";
        Restart = "on-failure";
        RestartSec = "5";
        Environment = [
          "XDG_RUNTIME_DIR=%t"
          "WAYLAND_DISPLAY=wayland-0"
        ];
      };

      Install = {
        WantedBy = [ "default.target" ];
      };
    };
  };
}
