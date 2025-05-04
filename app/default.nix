{
  crane,
  pkgs,
  rev,
}:

let
  toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  crane' = (crane.mkLib pkgs).overrideToolchain toolchain;

in
crane'.buildPackage {
  src = ./.;

  cargoVendorDir = crane'.vendorMultipleCargoDeps {
    inherit (crane'.findCargoFiles ./.) cargoConfigs;

    cargoLockList = [
      ./Cargo.lock
      "${toolchain.passthru.availableComponents.rust-src}/lib/rustlib/src/rust/library/Cargo.lock"
    ];

    overrideVendorCargoPackage =
      p: drv:
      if p.name == "russh-cryptovec" then
        drv.overrideAttrs (_: {
          patches = [
            # mlock() and munlock() are not available in NixOS containers
            ./nix/patches/russh-cryptovec-mlock.patch
          ];
        })
      else
        drv;
  };

  cargoExtraArgs = "-p kartoffels";
  CARGO_PROFILE = "dist";
  KARTOFFELS_REV = rev;

  __darwinAllowLocalNetworking = true;
}
