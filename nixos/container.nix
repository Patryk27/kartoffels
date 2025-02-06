{ self, nixpkgs }:

nixpkgs.lib.nixosSystem {
  system = "x86_64-linux";

  modules = [
    self.nixosModules.default

    (
      { pkgs, ... }:
      {
        boot = {
          isContainer = true;
        };

        networking = {
          firewall = {
            enable = false;
          };
        };

        services = {
          kartoffels = {
            enable = true;

            app = {
              secret = "demo";
            };

            nginx = {
              enable = true;
            };
          };
        };

        system = {
          stateVersion = "24.05";
        };
      }
    )
  ];
}
