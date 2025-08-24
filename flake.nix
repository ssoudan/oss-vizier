{
  description = "Env";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system: 
    let 
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
            inherit system overlays;
        };
        rustVersion = "latest";
        rustChannel = "nightly";
        #rustChannel = "stable";
        #rustVersion = "1.88.0";
        rust = pkgs.rust-bin.${rustChannel}.${rustVersion}.default.override {
        extensions = [
            "rust-src" # for rust-analyzer
        ];
  };
    in {

          devShells.default = pkgs.mkShell {
          buildInputs = [
          rust
          ] ++ (with pkgs; [
                  llvmPackages.bintools
                  bashInteractive 
                  python3
                  protobuf
                  just
                  rust-analyzer
                  rustc
                  cargo-edit
                  cargo-deny
          ] ++  # if darwin
                (if system == "aarch64-darwin" then [
                  pkgs.libiconv
                ] else [])
            );
          };
    });
}