{ crane, pkgs }:

let
  crane' = (crane.mkLib pkgs).overrideToolchain (
    pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml
  );

  src = crane'.cleanCargoSource ./.;

in
crane'.buildPackage {
  inherit src;

  # TODO
  doCheck = false;

  cargoBuildCommand = "cargo build -p kartoffels-server --release";

  cargoArtifacts = crane'.buildDepsOnly {
    inherit src;
  };
}
