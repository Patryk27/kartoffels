{ napalm, pkgs }:

let
  napalm' = pkgs.callPackage napalm { };

in
napalm'.buildPackage ./. {
  installPhase = ''
    npm run build
    mv dist $out
  '';

  VITE_HTTP_URL = "/api";
  VITE_WS_URL = "/api";
}
