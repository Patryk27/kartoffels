{ self }:
{ config, lib, pkgs, ... }:
with lib;

let
  cfg = config.services.kartoffels;

in
{
  options = {
    services.kartoffels = {
      enable = mkEnableOption "Kartoffels, a robot combat arena";

      backend = {
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.backend;
        };

        data = mkOption {
          type = types.str;
          default = "/var/lib/kartoffels";
        };

        http = mkOption {
          type = types.nullOr types.str;
          default = "0.0.0.0:81";
        };

        ssh = mkOption {
          type = types.nullOr types.str;
          default = "0.0.0.0:22";
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
        enable = mkEnableOption "nginx proxy";

        package = mkOption {
          type = types.package;
          default = pkgs.nginx;
        };

        listen = {
          addr = mkOption {
            type = types.str;
            default = "0.0.0.0";
          };

          port = mkOption {
            type = types.int;
            default = 80;
          };
        };
      };
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [
      cfg.backend.package
    ];

    services.nginx = mkIf cfg.nginx.enable {
      enable = true;

      virtualHosts = {
        default = {
          default = true;

          listen = [
            {
              addr = cfg.nginx.listen.addr;
              port = cfg.nginx.listen.port;
            }
          ];

          locations = {
            "/" = {
              root = "${cfg.frontend.package}";
            };

            "/api" = {
              proxyPass = "http://${cfg.backend.http}";
              proxyWebsockets = true;
            };
          };
        };
      };
    };

    systemd.services.kartoffels = {
      script = ''
        mkdir -p "${cfg.backend.data}"

        ${cfg.backend.package}/bin/kartoffels serve \
            '${cfg.backend.data}' \
            ${optionalString (cfg.backend.http != null) "--http ${cfg.backend.http}"} \
            ${optionalString (cfg.backend.ssh != null) "--ssh ${cfg.backend.ssh}"} \
            ${optionalString cfg.backend.debug "--debug"}
      '';

      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
    };
  };
}
