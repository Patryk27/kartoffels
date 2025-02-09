{
  inputs = {
    crane = {
      url = "github:ipetkov/crane";
    };

    napalm = {
      url = "github:nix-community/napalm";

      inputs = {
        nixpkgs = {
          follows = "nixpkgs";
        };
      };
    };

    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-24.05";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";

      inputs = {
        nixpkgs = {
          follows = "nixpkgs";
        };
      };
    };

    utils = {
      url = "github:numtide/flake-utils";
    };
  };

  outputs =
    {
      self,
      crane,
      napalm,
      nixpkgs,
      rust-overlay,
      utils,
    }:
    let
      packages = utils.lib.eachDefaultSystem (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;

            overlays = [
              (import rust-overlay)
            ];
          };

          app = import ./app {
            inherit crane pkgs;

            rev = builtins.substring 0 7 (self.rev or "dirty");
          };

          web = import ./web {
            inherit napalm pkgs;
          };

        in
        {
          apps = {
            default = utils.lib.mkApp {
              drv = app;
            };

            web = utils.lib.mkApp {
              drv =
                let
                  web' = web.overrideAttrs (_: {
                    VITE_API_URL = "http://localhost:1313";
                  });

                in
                pkgs.writeShellScriptBin "web" ''
                  ${pkgs.python3}/bin/python -m http.server 5173 -d ${web'}
                '';
            };
          };

          packages = {
            inherit app web;

            default = pkgs.linkFarm "kartoffels" [
              {
                name = "app";
                path = app;
              }
              {
                name = "web";
                path = web;
              }
            ];
          };
        }
      );

      others = {
        nixosConfigurations = {
          container = import ./nixos/container.nix {
            inherit self nixpkgs;
          };
        };

        nixosModules = {
          default = import ./nixos/module.nix {
            inherit self;
          };
        };
      };

    in
    packages // others;
}
