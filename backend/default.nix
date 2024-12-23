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
  };

  nativeBuildInputs = with pkgs; [
    just
  ];

  cargoExtraArgs = "-p kartoffels";
  CARGO_PROFILE = "dist";
  KARTOFFELS_REV = rev;

  __darwinAllowLocalNetworking = true;
}
