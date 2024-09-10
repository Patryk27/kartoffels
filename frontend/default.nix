{ napalm, pkgs }:

let
  napalm' = pkgs.callPackage napalm { };

in
napalm'.buildPackage ./. {
  VITE_API_URL = "/api";

  installPhase = ''
    npm run build
    mv dist $out
  '';
}
