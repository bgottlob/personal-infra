{
  description = "A basic flake with a shell";
  inputs.nixpkgs.url = "nixpkgs/23.05";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in
      with pkgs;
      {
        devShells.default = mkShell {
          packages = [
            jsonnet
            jsonnet-bundler
            kubectl
            kubernetes-helm
            openssl
            postgresql
            sops
            tanka
            terraform
          ];
        };
      });
}
