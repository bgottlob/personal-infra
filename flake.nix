{
  description = "A basic flake with a shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system: let
      yokeVersion = "0.20.14";
      kopiumVersion = "0.23.0";
      overlays = [
        (import rust-overlay)
        (final: prev: {
           yoke = pkgs.buildGoModule rec {
             pname = "yoke";
             version = yokeVersion;
             src = pkgs.fetchFromGitHub {
               owner = "yokecd";
               repo = "yoke";
               tag = "v${yokeVersion}";
               hash = "sha256-08cVWYKfNsMYqAtS4SqSkBWQ8uHEEjraf7NYlAjJnyc=";
             };
	     vendorHash = "sha256-zQ/rLzhrN9TX8w+n62MfLPSXyCf37bS2y9IB4a7dyxg=";
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
               hash = "sha256-QEdUALde9BVRioUlu6Y/zz7tZ0/lLxcteWQD92x4kvI=";
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
      tofu = pkgs.writeShellScriptBin "tofu" ''
        exec ${pkgs.opentofu}/bin/tofu -chdir="$TOFU_ROOT" "$@"
      '';
      sops-seal = pkgs.rustPlatform.buildRustPackage {
        pname = "sops-seal";
        version = "0.1.0";
        src = builtins.path {
          name = "k8s-deployments";
          path = ./k8s-deployments;
          filter = path: type:
            builtins.baseNameOf path != "target";
        };
        cargoLock.lockFile = ./k8s-deployments/Cargo.lock;
        cargoBuildFlags = [ "--bin" "sops-seal" ];
        doCheck = false;
      };
    in
      with pkgs;
      {
        devShells.default = mkShell {
          packages = [
            cargo-generate
            cargo-wasi
            cmctl
            egctl
            jsonnet
            jsonnet-bundler
            just
            kopium
            kubectl
            kubernetes-helm
            kubeseal
            mermaid-cli
            openssl
            pkg-config
            postgresql
            s3cmd
            sops
            sops-seal
            tanka
            tofu
            velero
            yoke
          ];
          nativeBuildInputs = [rust];
        };
      });
}
