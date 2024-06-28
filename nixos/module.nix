{ self }:
{ config, lib, pkgs, ... }:
with lib;

let
  cfg = config.services.kartoffels;

in
{
  options = {
    services.kartoffels = {
      enable = mkEnableOption "Kartoffels, an online robot combat arena";

      backend = {
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.backend;
        };

        listen = mkOption {
          type = types.str;
          default = "127.0.0.1:8080";
        };

        data = mkOption {
          type = types.str;
          default = "/var/kartoffels/data";
        };

        secret = mkOption {
          type = types.nullOr types.str;
          default = null;
        };

        debug = mkOption {
          type = types.bool;
          default = false;
        };
      };

      frontend = {
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.frontend;
        };
      };

      nginx = {
        enable = mkEnableOption "nginx proxy for backend and frontend";

        package = mkOption {
          type = types.package;
          default = pkgs.nginx;
        };
      };
    };
  };

  config = mkIf cfg.enable {
    services.nginx = mkIf cfg.nginx.enable {
      enable = true;

      virtualHosts = {
        default = {
          default = true;

          listen = [
            # TODO make configurable
            { addr = "0.0.0.0"; port = 80; }
          ];

          locations = {
            "/" = {
              root = "${cfg.frontend.package}";
            };

            "/api/" = {
              proxyPass = "http://${cfg.backend.listen}/";
            };
          };
        };
      };
    };

    systemd.services.kartoffels-server = {
      script = ''
        mkdir -p "${cfg.backend.data}"

        ${cfg.backend.package}/bin/kartoffels-server \
            --listen ${cfg.backend.listen} \
            --data ${cfg.backend.data} \
            ${optionalString (cfg.backend.secret != null) "--secret '${cfg.backend.secret}'"} \
            ${optionalString cfg.backend.debug "--debug"}
      '';

      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
    };
  };
}
