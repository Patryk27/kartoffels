{ crane, pkgs }:

let
  toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  crane' = (crane.mkLib pkgs).overrideToolchain toolchain;

  # TODO use lib.cleanSourceWith
  src = ./.;

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
    inherit src cargoVendorDir;

    cargoBuildCommand = "cargo build -p kartoffels-server --release";
    cargoExtraArgs = "--workspace --exclude kartoffels-sandbox";
  };

  kartoffels-sandbox = crane'.buildPackage {
    inherit src cargoVendorDir;

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

      ${pkgs.removeReferencesTo}/bin/remove-references-to \
        -t ${cargoVendorDir} \
        $out/kartoffels_sandbox_bg.wasm
    '';
  };

  roberto = crane'.buildPackage {
    inherit src cargoVendorDir;

    # N.B. this is already defined in `.cargo/config.toml`, but Crane's
    #      mkDummySrc accidentally gets rid of it
    RUSTFLAGS = "-C link-arg=-T${./crates/kartoffel/misc/kartoffel.ld}";

    cargoCheckCommand = ":";
    cargoTestCommand = ":";

    cargoExtraArgs = builtins.concatStringsSep " " [
      "-p roberto"
      "-Z build-std"
      "-Z build-std-features=compiler-builtins-mem"
      "--target ${./misc/riscv64-kartoffel-bot.json}"
    ];

    postInstall = ''
      ${pkgs.removeReferencesTo}/bin/remove-references-to \
        -t ${toolchain} \
        $out/bin/roberto
    '';

  };
}
