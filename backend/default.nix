{ crane, pkgs }:

let
  toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  crane' = (crane.mkLib pkgs).overrideToolchain toolchain;

  src = ./.;

  cargoVendorDir = crane'.vendorMultipleCargoDeps {
    inherit (crane'.findCargoFiles ./.) cargoConfigs;

    cargoLockList = [
      ./Cargo.lock
      "${toolchain.passthru.availableComponents.rust-src}/lib/rustlib/src/rust/Cargo.lock"
    ];
  };

  mkBot = name:
    let
      pkg = crane'.buildPackage {
        inherit src cargoVendorDir;

        # N.B. this is already defined in `.cargo/config.toml`, but Crane's
        #      mkDummySrc accidentally gets rid of it
        RUSTFLAGS = "-C link-arg=-T${./crates/kartoffel/kartoffel.ld}";

        cargoCheckCommand = ":";
        cargoTestCommand = ":";

        cargoExtraArgs = builtins.concatStringsSep " " [
          "-p bot-${name}"
          "-Z build-std"
          "-Z build-std-features=compiler-builtins-mem"
          "--target ${./riscv64-kartoffel-bot.json}"
        ];

        postInstall = ''
          ${pkgs.removeReferencesTo}/bin/remove-references-to \
            -t ${toolchain} \
            $out/bin/bot-${name}
        '';
      };

    in
    "${pkg}/bin/bot-${name}";

in
crane'.buildPackage {
  inherit src cargoVendorDir;

  cargoExtraArgs = "-p kartoffels";
  CARGO_PROFILE = "dist";
  KARTOFFELS_BOT_DUMMY = mkBot "dummy";
  KARTOFFELS_BOT_ROBERTO = mkBot "roberto";
}
