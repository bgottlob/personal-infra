{
  description = "A basic flake with a shell";

  inputs = {
    nixpkgs.url = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system: let
      yokeVersion = "0.15.6";
      overlays = [
        (import rust-overlay)
        (final: prev: {
           yoke = pkgs.buildGoModule rec {
             pname = "yoke";
             version = yokeVersion;
             src = pkgs.fetchFromGitHub {
               owner = "yokecd";
               repo = "yoke";
               tag = "yokecd/${yokeVersion}";
               hash = "sha256-W8ouqrIB4RHStZ2Yhn+wzq4emMkhjA8Vi9Cc0p46Emo=";
             };
	     vendorHash = "sha256-/4IINHW+T8vGBP166rdSLickki1Wh30/77K49hlwLM4=";
	     # Tests for this build a kind cluster, so skip that
	     doCheck = false;
           };
         })
      ];
      pkgs = import nixpkgs { inherit system overlays; };
      rust = pkgs.rust-bin.stable.latest.default.override {
        targets = [ "wasm32-wasip1" ];
      };
    in
      with pkgs;
      {
        devShells.default = mkShell {
          packages = [
            cargo-wasi
            jsonnet
            jsonnet-bundler
            kubectl
            kubernetes-helm
            openssl
            opentofu
            postgresql
            sops
            tanka
            velero
            yoke
            pkg-config
          ];
          nativeBuildInputs = [rust];
        };
      });
}
