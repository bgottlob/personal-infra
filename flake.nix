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
      kopiumVersion = "0.22.5";
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

        (final: prev: {
         kopium = pkgs.rustPlatform.buildRustPackage rec {
           pname = "kopium";
           name = "kopium";
           verison = kopiumVersion;
           src = pkgs.fetchFromGitHub {
               owner = "kube-rs";
               repo = "kopium";
               tag = "${kopiumVersion}";
               hash = "sha256-zYmb+HxwEKEnzdqAzvki5M+NA2fGP174pRkU6B4WmZI=";
           };
           cargoLock.lockFile = src + "/Cargo.lock";
           # todo - get tests working
           # https://github.com/NixOS/nixpkgs/blob/master/doc/languages-frameworks/rust.section.md#disabling-package-tests-disabling-package-tests
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
            just
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
            kopium
            cargo-generate
            s3cmd
            egctl
          ];
          nativeBuildInputs = [rust];
        };
      });
}
