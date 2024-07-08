{ crane, pkgs }:

let
  toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  crane' = (crane.mkLib pkgs).overrideToolchain toolchain;

in
{
  kartoffels-server = crane'.buildPackage {
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
  };

  kartoffels-sandbox = crane'.buildPackage {
    src = ./.;
    cargoCheckCommand = ":";
    cargoBuildCommand = "HOME=$(mktemp -d fake-homeXXXX) wasm-pack build ./crates/kartoffels-sandbox --target web";
    cargoTestCommand = ":";
    CARGO_BUILD_TARGET = "wasm32-unknown-unknown";

    installPhase = ''
      mkdir $out
      cp -avr ./crates/kartoffels-sandbox/pkg/* $out
    '';

    buildInputs = with pkgs; [
      binaryen
      wasm-bindgen-cli
      wasm-pack
    ];
  };
}
