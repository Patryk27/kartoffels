{ crane, pkgs }:

let
  toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  crane' = (crane.mkLib pkgs).overrideToolchain toolchain;

  cargoVendorDir = crane'.vendorMultipleCargoDeps {
    inherit (crane'.findCargoFiles ./.) cargoConfigs;

    cargoLockList = [
      ./Cargo.lock
      "${toolchain.passthru.availableComponents.rust-src}/lib/rustlib/src/rust/Cargo.lock"
    ];
  };

in
rec
{
  kartoffels-server = crane'.buildPackage {
    inherit cargoVendorDir;

    src = ./.;
    cargoBuildCommand = "cargo build -p kartoffels-server --release";

    # N.B. the server itself doesn't rely on roberto, but because we can't
    #      exclude a crate from a workspace, we have to provide it anyway
    KARTOFFELS_ROBERTO = "${roberto}/bin/roberto";
  };

  kartoffels-sandbox = crane'.buildPackage {
    inherit cargoVendorDir;

    src = ./.;

    buildInputs = with pkgs; [
      binaryen
      wasm-bindgen-cli
      wasm-pack
    ];

    CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
    KARTOFFELS_ROBERTO = "${roberto}/bin/roberto";

    cargoCheckCommand = ":";
    cargoTestCommand = ":";

    cargoBuildCommand = ''
      HOME=$(mktemp -d fake-homeXXXX) \
          wasm-pack build \
              ./crates/kartoffels-sandbox \
              --target web \
    '';

    installPhase = ''
      mkdir $out
      cp -avr ./crates/kartoffels-sandbox/pkg/* $out
    '';
  };

  roberto = crane'.buildPackage {
    inherit cargoVendorDir;

    src = ./.;
    cargoCheckCommand = ":";
    cargoTestCommand = ":";

    cargoExtraArgs = builtins.concatStringsSep " " [
      "-p roberto"
      "-Z build-std"
      "-Z build-std-features=compiler-builtins-mem"
      "--target ${./misc/riscv64-kartoffel-bot.json}"
    ];

    # N.B. this is already defined within `.cargo/config.toml`, but Crane seems
    #      to ignore it - not sure why
    RUSTFLAGS = "-C link-arg=-T${./crates/kartoffel/misc/kartoffel.ld}";
  };
}
