{
  crane,
  pkgs,
  rev,
}:

let
  inherit (pkgs) lib;

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

    overrideVendorGitCheckout =
      ps: drv:
      if lib.any (p: lib.hasPrefix "git+https://github.com/Patryk27/russh" p.source) ps then
        drv.overrideAttrs (_: {
          patches = [
            ./nix/patches/russh-cryptovec-mlock.patch
          ];
        })
      else
        drv;
  };

  nativeBuildInputs = with pkgs; [
    just
  ];

  cargoExtraArgs = "-p kartoffels";
  CARGO_PROFILE = "dist";
  KARTOFFELS_REV = rev;

  __darwinAllowLocalNetworking = true;
}
