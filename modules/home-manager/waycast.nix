{
  config,
  lib,
  pkgs,
  ...
}:

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
      description = "The Waycast package to use";
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
  };

  config = mkIf cfg.enable {
    home.packages = [ cfg.package ];

    xdg.configFile = mkIf (cfg.settings != { }) {
      "waycast/waycast.toml".source = toToml cfg.settings;
    };

    # Ensure cache and data dirs exist to avoid runtime errors in the future
    home.file."${config.xdg.cacheHome}/waycast/.keep".text = "";
    home.file."${config.xdg.dataHome}/waycast/.keep".text = "";

    # Install waycast icons to XDG_DATA_HOME
    home.file."${config.xdg.dataHome}/waycast/icons" = {
      source = "${cfg.package}/share/waycast/icons";
      recursive = true;
    };

    # Enable waycast daemon as systemd user service
    systemd.user.services.waycast-daemon = {
      Unit = {
        Description = "Waycast application launcher daemon";
        Documentation = "https://waycast.dev";
        After = [ "graphical-session.target" ];
        PartOf = [ "graphical-session.target" ];
      };

      Service = {
        Type = "simple";
        ExecStart = lib.getExe cfg.package;
        Restart = "always";
        RestartSec = "1s";
        Environment = [ "XDG_RUNTIME_DIR=%t" ];
      };

      Install = {
        WantedBy = [ "graphical-session.target" ];
      };
    };
  };
}
