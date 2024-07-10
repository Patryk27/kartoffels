{ napalm, pkgs, kartoffels-sandbox }:

let
  napalm' = pkgs.callPackage napalm { };

in
napalm'.buildPackage ./. {
  VITE_HTTP_URL = "/api";
  VITE_WS_URL = "/api";

  buildInputs = with pkgs; [
    nodePackages.prettier
  ];

  installPhase = ''
    rm node_modules/kartoffels-sandbox
    ln -s ${kartoffels-sandbox} node_modules/kartoffels-sandbox

    npm exec vue-tsc
    prettier . --check
    npm run build
    mv dist $out
  '';
}
