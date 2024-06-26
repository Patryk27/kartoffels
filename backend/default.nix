{ crane, pkgs }:

let
  toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  crane' = (crane.mkLib pkgs).overrideToolchain toolchain;

in
crane'.buildPackage {
  src = ./.;
  cargoBuildCommand = "cargo build -p kartoffels-server --release";

  cargoVendorDir = crane'.vendorMultipleCargoDeps {
    inherit (crane'.findCargoFiles ./.) cargoConfigs;

    cargoLockList = [
      ./Cargo.lock

      # Over kartoffels-vm's tests we build a couple of RISC-V test fixtures
      # using `cargo -Z build-std`, so we need to pull compiler_builtins and
      # other libraries that rust-src depends on:
      "${toolchain.passthru.availableComponents.rust-src}/lib/rustlib/src/rust/Cargo.lock"
    ];
  };
}
